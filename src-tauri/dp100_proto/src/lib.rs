use hidapi::{HidApi, HidDevice};

const VID: u16 = 0x2E3C;
const PID: u16 = 0xAF01;

// ─── CRC-16/Modbus ──────────────────────────────────────────

fn crc16(data: &[u8]) -> u16 {
    let mut crc: u16 = 0xFFFF;
    for &b in data {
        crc ^= b as u16;
        for _ in 0..8 {
            if crc & 1 == 1 {
                crc >>= 1;
                crc ^= 0xA001;
            } else {
                crc >>= 1;
            }
        }
    }
    crc
}

// ─── Data types ─────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct DeviceInfo {
    pub name: String,
    pub hardware_version: f64,
    pub firmware_version: f64,
    pub serial: String,
    pub year: u16,
    pub month: u8,
    pub day: u8,
}

#[derive(Debug, Clone)]
pub struct Telemetry {
    pub vin: u16,
    pub vout: u16,
    pub iout: u16,
    pub vo_max: u16,
    pub temp1: u16,
    pub temp2: i16,
    pub dc_5v: u16,
    pub out_mode: u8,
    pub work_st: u8,
}

#[derive(Debug, Clone)]
pub struct Preset {
    pub index: u8,
    pub state: u8,
    pub voltage: u16,
    pub current: u16,
    pub ovp: u16,
    pub ocp: u16,
}

#[derive(Debug, Clone)]
pub struct SystemSettings {
    pub otp: u16,
    pub opp: u16,
    pub backlight: u8,
    pub volume: u8,
    pub reverse_protection: bool,
    pub auto_output: bool,
}

// ─── Device handle ──────────────────────────────────────────

pub type LogCallback = Box<dyn Fn(&str) + Send>;

pub struct DP100 {
    device: HidDevice,
    seq: u8,
    log_cb: Option<LogCallback>,
}

impl DP100 {
    pub fn device_count() -> Result<usize, String> {
        let api = HidApi::new().map_err(|e| format!("HID init: {e}"))?;
        Ok(api
            .device_list()
            .filter(|d| d.vendor_id() == VID && d.product_id() == PID)
            .count())
    }

    pub fn open(index: usize) -> Result<Self, String> {
        let api = HidApi::new().map_err(|e| format!("HID init: {e}"))?;
        let info = api
            .device_list()
            .filter(|d| d.vendor_id() == VID && d.product_id() == PID)
            .nth(index)
            .ok_or("DP100 not found. Check USB connection.")?;
        let device = info
            .open_device(&api)
            .map_err(|e| format!("Failed to open: {e}"))?;
        Ok(Self { device, seq: 0, log_cb: None })
    }

    pub fn set_log_callback(&mut self, cb: LogCallback) {
        self.log_cb = Some(cb);
    }

    fn log(&self, msg: &str) {
        if let Some(ref cb) = self.log_cb {
            cb(msg);
        }
    }

    fn next_seq(&mut self) -> u8 {
        self.seq = self.seq.wrapping_add(1);
        self.seq
    }

    pub fn flush(&self) {
        let mut buf = [0u8; 64];
        while self.device.read_timeout(&mut buf, 5).unwrap_or(0) > 0 {}
    }

    fn write(&self, frame: &[u8; 64]) -> bool {
        self.device.write(frame).is_ok()
    }

    fn read(&self, timeout_ms: i32) -> Option<[u8; 64]> {
        let mut buf = [0u8; 64];
        match self.device.read_timeout(&mut buf, timeout_ms) {
            Ok(len) if len > 0 => Some(buf),
            _ => None,
        }
    }

    fn build_frame(&self, opcode: u8, seq: u8, content: &[u8]) -> [u8; 64] {
        assert!(content.len() <= 58, "content too long for HID frame: {}", content.len());
        let mut frame = [0u8; 64];
        frame[0] = 0xFB;
        frame[1] = opcode;
        frame[2] = seq;
        frame[3] = content.len() as u8;
        frame[4..4 + content.len()].copy_from_slice(content);
        let crc = crc16(&frame[..4 + content.len()]);
        frame[4 + content.len()] = (crc & 0xFF) as u8;
        frame[4 + content.len() + 1] = (crc >> 8) as u8;
        frame
    }

    /// Send frame and wait for response with matching opcode.
    /// Retries up to 5 times, skipping responses with wrong opcode (e.g. stale polling).
    fn session(&self, frame: &[u8; 64], expected_opcode: u8) -> Option<[u8; 64]> {
        self.flush();
        let clen = frame[3] as usize;
        self.log(&format!("TX op={:02x} seq={:02x} len={} data={:02x?}",
            frame[1], frame[2], clen, &frame[4..4+clen.min(16)]));
        if !self.write(frame) {
            self.log("TX write failed");
            return None;
        }
        for attempt in 0..5 {
            if let Some(buf) = self.read(200) {
                if buf[0] == 0xFA && buf[1] == expected_opcode {
                    let rlen = buf[3] as usize;
                    self.log(&format!("RX op={:02x} len={} data={:02x?}",
                        buf[1], rlen, &buf[4..4+rlen.min(16)]));
                    return Some(buf);
                }
                if attempt == 0 {
                    self.log(&format!("RX skip op={:02x} (want {:02x})", buf[1], expected_opcode));
                }
            }
        }
        self.log(&format!("RX timeout (want op={:02x})", expected_opcode));
        None
    }

    // ─── Device Info (opcode 0x10) ──────────────────────────

    pub fn device_info(&mut self) -> Option<DeviceInfo> {
        let seq = self.next_seq();
        let frame = self.build_frame(0x10, seq, &[]);
        let resp = self.session(&frame, 0x10)?;
        if (resp[3] as usize) < 40 {
            return None;
        }
        let name = String::from_utf8_lossy(&resp[4..20])
            .trim_end_matches(|c: char| c == '\0' || !c.is_ascii_graphic())
            .to_string();
        let serial = resp[28..36]
            .iter()
            .map(|b| format!("{b:02X}"))
            .collect::<String>();
        Some(DeviceInfo {
            name,
            hardware_version: u16::from_le_bytes([resp[20], resp[21]]) as f64 / 10.0,
            firmware_version: u16::from_le_bytes([resp[22], resp[23]]) as f64 / 10.0,
            serial,
            year: u16::from_le_bytes([resp[40], resp[41]]),
            month: resp[42],
            day: resp[43],
        })
    }

    // ─── Telemetry / Basic Info (opcode 0x30) ───────────────

    pub fn basic_info(&mut self) -> Option<Telemetry> {
        let seq = self.next_seq();
        let frame = self.build_frame(0x30, seq, &[]);
        let resp = self.session(&frame, 0x30)?;
        if (resp[3] as usize) < 16 {
            return None;
        }
        let d = &resp[4..];
        Some(Telemetry {
            vin: u16::from_le_bytes([d[0], d[1]]),
            vout: u16::from_le_bytes([d[2], d[3]]),
            iout: u16::from_le_bytes([d[4], d[5]]),
            vo_max: u16::from_le_bytes([d[6], d[7]]),
            temp1: u16::from_le_bytes([d[8], d[9]]),
            temp2: i16::from_le_bytes([d[10], d[11]]),
            dc_5v: u16::from_le_bytes([d[12], d[13]]),
            out_mode: d[14],
            work_st: d[15],
        })
    }

    // ─── Presets / Basic Set (opcode 0x35) ───────────────────

    fn parse_preset(resp: &[u8; 64]) -> Option<Preset> {
        if (resp[3] as usize) < 10 {
            return None;
        }
        let d = &resp[4..];
        Some(Preset {
            index: d[0] & 0x0F,
            state: d[1],
            voltage: u16::from_le_bytes([d[2], d[3]]),
            current: u16::from_le_bytes([d[4], d[5]]),
            ovp: u16::from_le_bytes([d[6], d[7]]),
            ocp: u16::from_le_bytes([d[8], d[9]]),
        })
    }

    /// Read preset by index (0-9). Index 0x80 = currently active preset.
    pub fn read_preset(&mut self, index: u8) -> Option<Preset> {
        let seq = self.next_seq();
        let frame = self.build_frame(0x35, seq, &[index]);
        let resp = self.session(&frame, 0x35)?;
        Self::parse_preset(&resp)
    }

    /// Read the currently active preset.
    pub fn current_preset(&mut self) -> Option<Preset> {
        self.read_preset(0x80)
    }

    /// Apply values to the active output (does NOT save to preset storage).
    /// Flag 0x20: set output.
    pub fn set_output(
        &mut self,
        enabled: bool,
        voltage: u16,
        current: u16,
        ovp: u16,
        ocp: u16,
    ) -> bool {
        let seq = self.next_seq();
        let mut content = [0u8; 10];
        content[0] = 0x20;
        content[1] = if enabled { 1 } else { 0 };
        content[2..4].copy_from_slice(&voltage.to_le_bytes());
        content[4..6].copy_from_slice(&current.to_le_bytes());
        content[6..8].copy_from_slice(&ovp.to_le_bytes());
        content[8..10].copy_from_slice(&ocp.to_le_bytes());
        let frame = self.build_frame(0x35, seq, &content);
        if let Some(resp) = self.session(&frame, 0x35) {
            return resp[3] == 1 && resp[4] == 1;
        }
        false
    }

    /// Save values to a preset slot (flash storage). Does NOT apply to output.
    /// Flag 0x40: save to preset.
    pub fn save_preset(
        &mut self,
        index: u8,
        voltage: u16,
        current: u16,
        ovp: u16,
        ocp: u16,
    ) -> bool {
        let seq = self.next_seq();
        let mut content = [0u8; 10];
        content[0] = 0x40 + index;
        content[1] = 0;
        content[2..4].copy_from_slice(&voltage.to_le_bytes());
        content[4..6].copy_from_slice(&current.to_le_bytes());
        content[6..8].copy_from_slice(&ovp.to_le_bytes());
        content[8..10].copy_from_slice(&ocp.to_le_bytes());
        let frame = self.build_frame(0x35, seq, &content);
        if let Some(resp) = self.session(&frame, 0x35) {
            return resp[3] == 1 && resp[4] == 1;
        }
        false
    }

    /// Activate a preset (switch to it). Loads preset values to output.
    /// Flag 0x80: activate preset.
    pub fn activate_preset(
        &mut self,
        index: u8,
        state: u8,
        voltage: u16,
        current: u16,
        ovp: u16,
        ocp: u16,
    ) -> bool {
        let seq = self.next_seq();
        let mut content = [0u8; 10];
        content[0] = 0x80 + index;
        content[1] = state;
        content[2..4].copy_from_slice(&voltage.to_le_bytes());
        content[4..6].copy_from_slice(&current.to_le_bytes());
        content[6..8].copy_from_slice(&ovp.to_le_bytes());
        content[8..10].copy_from_slice(&ocp.to_le_bytes());
        let frame = self.build_frame(0x35, seq, &content);
        if let Some(resp) = self.session(&frame, 0x35) {
            return resp[3] == 1 && resp[4] == 1;
        }
        false
    }

    // ─── System Settings (opcode 0x40) ──────────────────────

    /// Read system settings (len=0 → read).
    pub fn system_settings(&mut self) -> Option<SystemSettings> {
        let seq = self.next_seq();
        let frame = self.build_frame(0x40, seq, &[]);
        let resp = self.session(&frame, 0x40)?;
        if (resp[3] as usize) < 8 {
            return None;
        }
        let d = &resp[4..];
        Some(SystemSettings {
            otp: u16::from_le_bytes([d[0], d[1]]),
            opp: u16::from_le_bytes([d[2], d[3]]),
            backlight: d[4],
            volume: d[5],
            reverse_protection: d[6] != 0,
            auto_output: d[7] != 0,
        })
    }

    /// Write system settings (len=8 → write).
    pub fn set_system_settings(&mut self, settings: &SystemSettings) -> bool {
        let seq = self.next_seq();
        let mut content = [0u8; 8];
        content[0..2].copy_from_slice(&settings.otp.to_le_bytes());
        content[2..4].copy_from_slice(&settings.opp.to_le_bytes());
        content[4] = settings.backlight;
        content[5] = settings.volume;
        content[6] = if settings.reverse_protection { 1 } else { 0 };
        content[7] = if settings.auto_output { 1 } else { 0 };
        let frame = self.build_frame(0x40, seq, &content);
        if let Some(resp) = self.session(&frame, 0x40) {
            return resp[3] == 1 && resp[4] == 1;
        }
        false
    }

    // ─── Scanning (opcode 0x50) ───────────────────────────

    /// Start a voltage or current scan.
    /// scan_mode: 0 = current scan (CC), 1 = voltage scan (CV)
    /// out_val: fixed V (for current scan) or fixed I (for voltage scan), in mV or mA
    /// start/end/step: scan range in mV or mA
    /// delay_ms: time per step
    pub fn start_scan(
        &mut self,
        scan_mode: u8,
        out_val: u16,
        start: u16,
        end: u16,
        step: u16,
        delay_ms: u16,
    ) -> bool {
        let seq = self.next_seq();
        let mut content = [0u8; 12];
        content[0] = 0x01; // on_off = start
        content[1..3].copy_from_slice(&delay_ms.to_le_bytes());
        content[3..5].copy_from_slice(&out_val.to_le_bytes());
        content[5] = scan_mode;
        content[6..8].copy_from_slice(&start.to_le_bytes());
        content[8..10].copy_from_slice(&end.to_le_bytes());
        content[10..12].copy_from_slice(&step.to_le_bytes());
        let frame = self.build_frame(0x50, seq, &content);
        if let Some(resp) = self.session(&frame, 0x50) {
            return resp[3] == 1 && resp[4] == 1;
        }
        false
    }

    /// Stop a running scan.
    pub fn stop_scan(&mut self) -> bool {
        let seq = self.next_seq();
        let content = [0u8; 12]; // on_off = 0 = stop
        let frame = self.build_frame(0x50, seq, &content);
        if let Some(resp) = self.session(&frame, 0x50) {
            return resp[3] == 1 && resp[4] == 1;
        }
        false
    }

    // ─── Serial Output (opcode 0x55) ────────────────────────

    /// Start serial (sequence) output.
    /// ser_start/ser_end: step indices (1-200)
    /// voltage/current: output values in mV/mA
    /// cycle_times: number of cycles (0 = infinite?)
    /// delay_ms: time per step
    pub fn start_serial(
        &mut self,
        ser_start: u8,
        ser_end: u8,
        voltage: u16,
        current: u16,
        cycle_times: u8,
        delay_ms: u16,
    ) -> bool {
        let seq = self.next_seq();
        let mut content = [0u8; 10];
        content[0] = 0x01; // on_off = start
        content[1..3].copy_from_slice(&delay_ms.to_le_bytes());
        content[3] = ser_start;
        content[4] = ser_end;
        content[5..7].copy_from_slice(&current.to_le_bytes());
        content[7..9].copy_from_slice(&voltage.to_le_bytes());
        content[9] = cycle_times;
        let frame = self.build_frame(0x55, seq, &content);
        if let Some(resp) = self.session(&frame, 0x55) {
            return resp[3] == 1 && resp[4] == 1;
        }
        false
    }

    /// Stop serial output.
    pub fn stop_serial(&mut self) -> bool {
        let seq = self.next_seq();
        let content = [0u8; 10]; // on_off = 0 = stop
        let frame = self.build_frame(0x55, seq, &content);
        if let Some(resp) = self.session(&frame, 0x55) {
            return resp[3] == 1 && resp[4] == 1;
        }
        false
    }

    // ─── Protection (opcode 0x35, flag 0x40 on current) ─────

    pub fn set_protection(&mut self, ovp: u16, ocp: u16) -> bool {
        let seq = self.next_seq();
        let mut content = [0u8; 10];
        content[0] = 0x40;
        content[6..8].copy_from_slice(&ovp.to_le_bytes());
        content[8..10].copy_from_slice(&ocp.to_le_bytes());
        let frame = self.build_frame(0x35, seq, &content);
        if let Some(resp) = self.session(&frame, 0x35) {
            return resp[3] == 1 && resp[4] == 1;
        }
        false
    }
}
