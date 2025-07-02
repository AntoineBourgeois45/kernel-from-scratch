pub unsafe fn draw_idle_frame(frame: usize) {
        const ART: [&str; 7] = [
            "    ###    ####",
            "   ####   ##  ##",
            "  ## ##       ##    Rust Kernel from scratch",
            " ##  ##     ###",
            " #######   ##       Version 0.2.1",
            "     ##   ##  ##",
            "     ##   ######",
        ];
        let rainbow = [
            VgaColor::Red,
            VgaColor::LightRed,
            VgaColor::LightBrown,
            VgaColor::LightGreen,
            VgaColor::LightCyan,
            VgaColor::LightBlue,
            VgaColor::Magenta,
            VgaColor::Cyan,
        ];
        let art_h = ART.len();
        let art_w = ART.iter().map(|l| l.len()).max().unwrap_or(0);

        let buf = &terminal().screen_buffers[0];

        for (dy, &line) in ART.iter().enumerate() {
            for (dx, b) in line.bytes().enumerate() {
                if b != b' ' {
                    let color = rainbow[(dx + dy + frame) % rainbow.len()];
                    let code  = vga_entry(b, vga_entry_color(color, VgaColor::Black));
                    let idx = dy * VGA_WIDTH + dx;
                    buf[idx] = code;
                }
            }
        }

        if self.current_screen == 0 {
            self.refresh_screen();
        }
    }