/*This is yet another emulator. It doesnt aim to be fastest optimized or anything
Its educational for myself, maybe for someone else learning Rust and emulators.
Expect a lot of "unnecessary" comments variable names and overall structure.
*/
use rand::random;

pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HEIGHT: usize = 32;

const RAM_SIZE: usize = 32768;//4096;
const NUM_REGS: usize = 16;
const STACK_SIZE: usize = 16;
const NUM_KEYS: usize = 16;
const START_ADDR: u16 = 0x200;

const FONTSET_SIZE: usize = 80;
const FONTSET: [u8; FONTSET_SIZE] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80  // F
];

pub struct Emu {
    pc: u16, // program counter
    ram: [u8; RAM_SIZE],
    screen: [bool; SCREEN_HEIGHT * SCREEN_WIDTH],
    v_reg: [u8; NUM_REGS], // hex from V0 to VF
    i_reg: u16, // indexes the ram
    sp: u16, // stack pointer, og
    stack: [u16; STACK_SIZE],
    keys: [bool; NUM_KEYS],
    dt: u8, // delay timer
    st: u8, // sound timer
}

struct DecodedOp {
    d1: u8,
    d2: u8,
    d3: u8,
    d4: u8,
    nnn: u16,
    nn: u8,
    x: usize,
    y: usize,
}

impl DecodedOp {
    fn new(op: u16) -> Self {
        let d1 = ((op & 0xF000) >> 12) as u8;
        let d2 = ((op & 0x0F00) >> 8) as u8;
        let d3 = ((op & 0x00F0) >> 4) as u8;
        let d4 = ( op & 0x000F) as u8;

        DecodedOp {
            d1, d2, d3, d4,
            x: d2 as usize,
            y: d3 as usize,
            nn:  (op & 0x00FF) as u8,
            nnn: (op & 0x0FFF) as u16,
        }
    }
}

impl Emu {
    pub fn new() -> Self {
        let mut new_emu = Self {
            pc: START_ADDR,
            ram: [0; RAM_SIZE],
            screen: [false; SCREEN_WIDTH * SCREEN_HEIGHT],
            v_reg: [0; NUM_REGS],
            i_reg: 0,
            sp: 0,
            stack: [0; STACK_SIZE],
            keys: [false; NUM_KEYS],
            dt: 0,
            st: 0,
        };
        new_emu.ram[0..FONTSET_SIZE].copy_from_slice(&FONTSET);
        new_emu
    }

    pub fn reset(&mut self) {
        self.pc = START_ADDR;
        self.ram = [0; RAM_SIZE];
        self.screen = [false; SCREEN_WIDTH * SCREEN_HEIGHT];
        self.v_reg = [0; NUM_REGS];
        self.i_reg = 0;
        self.sp = 0;
        self.stack = [0; STACK_SIZE];
        self.keys = [false; NUM_KEYS];
        self.dt = 0;
        self.st = 0;
        self.ram[..FONTSET_SIZE].copy_from_slice(&FONTSET);
    }

    // push an u16 value into the stack
    pub fn push(&mut self, val: u16) {
        self.stack[self.sp as usize] = val;
        self.sp += 1;

    }

    pub fn pop(&mut self) -> u16 {
        // todo: we could underflow here, but this would be a bug in the game, so a crash is valid
        self.sp -= 1;
        self.stack[self.sp as usize]
    }

    pub fn tick_timers(&mut self) {
        if self.dt > 0 {
            self.dt -= 1;
        }

        if self.st > 0 {
            if self.st == 1 {
                // BEEP
            }
            self.st -= 1;
        }
    }

    pub fn get_display(&self) -> &[bool] {
        &self.screen
    }

    pub fn keypress(&mut self, idx: usize, pressed: bool) {
        // frontend will check if idx < NUM_KEYS
        self.keys[idx] = pressed;
    }

    pub fn load_rom(&mut self, data: &[u8]) {
        let start = START_ADDR as usize;
        let end = (START_ADDR as usize) + data.len();
        self.ram[start..end].copy_from_slice(data);
    }

    fn fetch(&mut self) -> u16 {
        // every opcode in chip8 is 4 hex digits => 2 bytes
        let higher_byte = self.ram[self.pc as usize] as u16;
        let lower_byte = self.ram[(self.pc + 1) as usize] as u16;
        let op = (higher_byte << 8) | lower_byte; // merge the two bytes
        self.pc += 2;
        op
    }

    pub fn tick(&mut self) {
        // Fetch
        let op = self.fetch();
        // Decode & execute
        self.execute(op);
    }

    fn execute(&mut self, op: u16) {
        let d_op = DecodedOp::new(op);

        match (d_op.d1, d_op.d2, d_op.d3, d_op.d4) {
            (0x0, 0x0, 0x0, 0x0) => return,
            (0x0, 0x0, 0xE, 0x0) => self.cls(),
            (0x0, 0x0, 0xE, 0xE) => self.ret(),

            (0x1, _, _, _) => self.jmp(d_op.nnn),
            (0x2, _, _, _) => self.call(d_op.nnn),
            (0x3, _, _, _) => self.skip_eq_nn(d_op.x, d_op.nn),
            (0x4, _, _, _) => self.skip_neq_nn(d_op.x, d_op.nn),
            (0x5, _, _, 0) => self.skip_x_eq_y(d_op.x, d_op.y),
            (0x6, _, _, _) => self.set_to(d_op.x, d_op.nn),
            (0x7, _, _, _) => self.add_to(d_op.x, d_op.nn),
            (0x8, _, _, 0) => self.load_reg(d_op.x, d_op.y),

            (0x8, _, _, 1) => self.or(d_op.x, d_op.y),
            (0x8, _, _, 2) => self.and(d_op.x, d_op.y),
            (0x8, _, _, 3) => self.xor(d_op.x, d_op.y),
            (0x8, _, _, 4) => self.add_with_carry(d_op.x, d_op.y),
            (0x8, _, _, 5) => self.sub_with_borrow(d_op.x, d_op.y),
            (0x8, _, _, 6) => self.shr(d_op.x),
            (0x8, _, _, 7) => self.sub_n(d_op.x, d_op.y),
            (0x8, _, _, 8) => self.shl(d_op.x),
            (0x8, _, _, 0xE) => self.super_shl(d_op.x, d_op.y), // 0x800E: some octo games require this version of shl 

            (0x9, _, _, 0) => self.skip_x_neq_y(d_op.x, d_op.y),
            (0xA, _, _, _) => self.load_i(d_op.nnn),
            (0xB, _, _, _) => self.jump_v0(d_op.nnn),
            (0xC, _, _, _) => self.rand(d_op.x, d_op.nn),

            (0xD, _, _, _) => self.draw_sprite(d_op.x, d_op.y, d_op.d4),
            (0xE, _, 9, 0xE) => self.skip_key_pressed(d_op.x),
            (0xE, _, 0xA, 1) => self.skip_key_not_pressed(d_op.x),
            (0xF, _, 0, 7) => self.get_delay(d_op.x),
            (0xF, _, 0, 0xA) => self.wait_key(d_op.x),

            (0xF, _, 1, 5) => self.set_delay(d_op.x),
            (0xF, _, 1, 8) => self.set_sound(d_op.x),
            (0xF, _, 1, 0xE) => self.add_i(d_op.x),
            (0xF, _, 2, 9) => self.load_font_addr_into_i(d_op.x),
            (0xF, _, 3, 3) => self.store_bcd(d_op.x),
            (0xF, _, 5, 5) => self.reg_dump(d_op.x),
            (0xF, _, 6, 5) => self.reg_load(d_op.x),

            (_, _, _, _) => unimplemented!("Unimplemented opcode: {}", op),
        }
    }

    fn cls(&mut self) {
        self.screen = [false; SCREEN_HEIGHT * SCREEN_WIDTH];
    }

    fn ret(&mut self) {
        // Return from subroutine. Entering a subroutine is pushing current address into the stack, then run the code
        // and get the address you were before entering the subroutine
        let ret_addr = self.pop();
        self.pc = ret_addr;
    }

    fn jmp(&mut self, to: u16) {
        self.pc = to;
    }

    fn call(&mut self, addr: u16) {
        // call subroutine. Push the address we have before entering the subroutine (so we can run ret opcode).
        // move the subroutine address into the program counter.
        self.push(self.pc);
        self.pc = addr;
    }

    fn skip_eq_nn(&mut self, x: usize, nn: u8) {
        if self.v_reg[x] == nn {
            self.pc += 2;
        }
    }

    fn skip_neq_nn(&mut self, x: usize, nn: u8) {
        if self.v_reg[x] != nn {
            self.pc += 2;
        }
    }

    fn skip_x_eq_y(&mut self, x: usize, y: usize) {
        if self.v_reg[x] == self.v_reg[y] {
            self.pc += 2;
        }
    }

    fn set_to(&mut self, x: usize, nn: u8) {
        self.v_reg[x] = nn;
    }

    fn add_to(&mut self, x: usize, nn: u8) {
        // could overflow the register, chip-8 didnt do anything about that in this instruction. It will panic in that case
        self.v_reg[x] = self.v_reg[x].wrapping_add(nn);
    }

    fn load_reg(&mut self, x: usize, y: usize) {
        self.v_reg[x] = self.v_reg[y];
    }

    fn or(&mut self, x: usize, y: usize) {
        self.v_reg[x] |= self.v_reg[y];
    }

    fn and(&mut self, x: usize, y: usize) {
        self.v_reg[x] &= self.v_reg[y];
    }

    fn xor(&mut self, x: usize, y: usize) {
        self.v_reg[x] ^= self.v_reg[y];
    }

    fn add_with_carry(&mut self, x: usize, y: usize) {
        let (new_x, carry )= self.v_reg[x].overflowing_add(self.v_reg[y]);
        self.v_reg[0xF] = if carry {1} else {0};
        self.v_reg[x] = new_x;
    }

    fn sub_with_borrow(&mut self, x: usize, y: usize) {
        let (new_x, borrow )= self.v_reg[x].overflowing_sub(self.v_reg[y]);
        self.v_reg[0xF] = if borrow {0} else {1}; // if an underflow occurs, it is set to 0
        self.v_reg[x] = new_x;
    }

    fn shr(&mut self, x: usize) {
        // single shift right, dropped bit goes into VF
        self.v_reg[0xF] = self.v_reg[x] & 1;
        self.v_reg[x] >>= 1;
    }

    fn sub_n(&mut self, x:usize, y:usize) {
        // VX = VY - VX
        let (new_vx, borrow) = self.v_reg[y].overflowing_sub(self.v_reg[x]);
        self.v_reg[0xF] = if borrow {0} else {1};
        self.v_reg[x] = new_vx;
    }

    fn shl(&mut self, x: usize) {
        // single shift left, dropped bit goes into VF
        self.v_reg[0xF] = (self.v_reg[x] >> 7) & 1;
        self.v_reg[x] <<= 1;
    }

    fn super_shl(&mut self, x: usize, y: usize) {
        // 0x8XYE: Shift Vy left by 1, store result in Vx
        // VF gets the most significant bit of Vy BEFORE the shift
        
        // Get the MSB from Vy (the source register)
        self.v_reg[0xF] = (self.v_reg[y] >> 7) & 1;
        
        // Shift Vy left and store result in Vx
        self.v_reg[x] = self.v_reg[y] << 1;
        
        // Note: Vy remains unchanged!
    }
    

    fn skip_x_neq_y(&mut self, x: usize, y: usize) {
        if self.v_reg[x] != self.v_reg[y] {
            self.pc += 2;
        }
    }

    fn load_i(&mut self, nnn: u16) {
        self.i_reg = nnn;
    }

    fn jump_v0(&mut self, nnn: u16) {
        self.pc = (self.v_reg[0] as u16) + nnn;
    }

    fn rand(&mut self, x:usize, nn: u8) {
        // VX = rand() & 0xNN
        let rng: u8 = random();
        self.v_reg[x] = rng & nn;
    }

    fn draw_sprite(&mut self, x: usize, y: usize, d: u8) {
        // sprites are always 8 pixels wide, but can be a variable number of pixels tall, from 1 to 16.
        // where the sprite begins in the screen
        let x_coord = self.v_reg[x] as u16;
        let y_coord = self.v_reg[y] as u16;

        let n_nows = d as u16;
        let mut flipped = false;

        for y_line in 0..n_nows {
            // sprites are stored in ram row by row beginning at the address stored in I
            let row_addr = self.i_reg + y_line as u16;
            let pixels = self.ram[row_addr as usize];

            for x_line in 0..8 {
                // the screen is already blank, so check if we need to flip the pixel (1), otherwise we keep it blank (0)
                if pixels >> (7 - x_line) & 1 == 1 { // shr the row by x_line (only 1 bit remain), check if this is 1
                    // pixels “wrap” to the other side of the screen if they overflow screen size
                    let x = (x_coord + x_line) as usize % SCREEN_WIDTH;
                    let y = (y_coord + y_line) as usize % SCREEN_HEIGHT;

                    let idx_1d = x + SCREEN_WIDTH * y;
                    // If the pixel you're about to draw is currently ON, record that a collision happened.
                    // this opcode sets VF to 1 if this collision happened. Thats's the spec.
                    flipped |= self.screen[idx_1d]; 
                    self.screen[idx_1d] ^= true; // flip the pixel
                }
            }
        }
        // set vf if collision happened
        if flipped {
            self.v_reg[0xF] = 1;
        } else {
            self.v_reg[0xF] = 0;
        }
    }
    

    fn skip_key_pressed(&mut self, x: usize) {
        // this feels weird written, but remember there isnt a way to give x directly, its stored somewhere.
        let key = self.keys[self.v_reg[x] as usize];
        if  key {
            self.pc += 2;
        }
    }

    fn skip_key_not_pressed(&mut self, x: usize) {
        // this feels weird written, but remember there isnt a way to give x directly, its stored somewhere.
        let key = self.keys[self.v_reg[x] as usize];
        if  !key {
            self.pc += 2;
        }
    }

    fn get_delay(&mut self, x: usize) {
        self.v_reg[x] = self.dt;
    }

    fn wait_key(&mut self, x: usize) {
        // blocks code from running until any key is pressed. This is done by reverting the self.pc to be back to call this.
        // it also stores the key index that was true in VX.
        let mut pressed = false;

        for i in 0..self.keys.len() {
            if self.keys[i] {
                self.v_reg[x] = i as u8;
                pressed = true;
                break;
            }
        }
        if !pressed {
            self.pc -= 2; // redo
            // The original CHIP-8 was interpreted by the COSMAC VIP.
            // Fx0A halted execution until a key pressed interrupt fired.
            // we dont have interruptions here, so this is the way to avoid async checks
        }
    }

    fn set_delay(&mut self, x: usize) {
        self.dt = self.v_reg[x];
    }

    fn set_sound(&mut self, x: usize) {
        self.st = self.v_reg[x];
    }

    fn add_i(&mut self, x: usize) {
        self.i_reg = self.i_reg.wrapping_add(self.v_reg[x] as u16);
    }

    fn load_font_addr_into_i(&mut self, x: usize) {
        // // expects vx to hold a val from 0 to 0xF
        // // our font sprites are 5 bytes each starting at 0 in our ram
        // self.i_reg = (self.v_reg[x] as u16) * 5;
        let c = self.v_reg[x] as u16;
        self.i_reg = c * 5;
    }

    fn store_bcd(&mut self, x: usize) {
        // I = BCD (Binary Coded Decimal) of VX 
        // Store the hundreds, tens, and ones digits of VX into memory at I, I+1, I+2.
        let i = self.i_reg as usize;
        let v = self.v_reg[x];
        self.ram[i] = v / 100;
        self.ram[i + 1] = (v / 10) % 10;
        self.ram[i + 2] = v % 10;
    }

    fn reg_dump(&mut self, x: usize) {
        // Stores V0 thru VX (inclusive) into RAM address starting at I
        let i = self.i_reg as usize;
        //let vx = self.v_reg[x] as usize;
        for index in 0..=x {
            self.ram[i + index] = self.v_reg[index];
        }
    }

    fn reg_load(&mut self, x: usize) {
        // Loads V0 thru VX (inclusive) from RAM address starting at I
        let i = self.i_reg as usize;
        // let vx = self.v_reg[x] as usize;
        for index in 0..=x {
            self.v_reg[index] = self.ram[i + index];
            
        }
    } 
}
