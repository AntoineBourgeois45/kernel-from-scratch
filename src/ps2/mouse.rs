use super::controller::PS2Controller;

#[derive(Debug, Clone, Copy)]
pub struct MouseEvent {
    pub x_movement: i16,
    pub y_movement: i16,
    pub left_button: bool,
    pub right_button: bool,
    pub middle_button: bool,
}

impl PS2Controller {
    pub fn init_mouse(&mut self) {
            self.send_command(0xA8);
            self.send_command(0x20);

            while !self.has_data() {}
            let mut config = self.read_data();

            config |= 0x02;
            config &= !0x20;

            self.send_command(0x60);
            self.send_data(config);

            self.send_command(0xD4);
            self.send_data(0xF4);
        }
    pub fn process_mouse_data(&mut self, data: u8) -> Option<MouseEvent> {
        self.mouse_packet[self.mouse_cycle as usize] = data;
        self.mouse_cycle += 1;

        if self.mouse_cycle == 3 {
            self.mouse_cycle = 0;

            let flags = self.mouse_packet[0];
            let x_movement = self.mouse_packet[1] as i16;
            let y_movement = self.mouse_packet[2] as i16;

            let x_movement = if flags & 0x10 != 0 {
                x_movement | 0xFF00u16 as i16
            } else {
                x_movement
            };

            let y_movement = if flags & 0x20 != 0 {
                y_movement | 0xFF00u16 as i16
            } else {
                y_movement
            };

            Some(MouseEvent {
                x_movement,
                y_movement,
                left_button: flags & 0x01 != 0,
                right_button: flags & 0x02 != 0,
                middle_button: flags & 0x04 != 0,
            })
        } else {
            None
        }
    }
}