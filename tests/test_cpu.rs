extern crate lib;
use lib::{Cpu, OPCODE_SIZE};

#[test]
fn test_op_00ee() {
    let mut cpu = Cpu {
        memory: [0; 4096],
        opcode: 0,
        v: [0; 16],
        i: 0,
        pc: 0,
        gfx: [0; (64 * 32)],
        delay_timer: 0,
        sound_timer: 0,
        stack: [0; 16],
        sp: 0,
    };
    cpu.initialize();

    cpu.sp = 1;
    cpu.stack[cpu.sp] = 0x201;
    let target = cpu.stack[cpu.sp]; // where pc should end up

    cpu.run_opcode(0x00EE);

    assert_eq!(target, cpu.pc);
}

#[test]
fn test_op_1nnn() {
    let mut cpu = Cpu {
        memory: [0; 4096],
        opcode: 0,
        v: [0; 16],
        i: 0,
        pc: 0,
        gfx: [0; (64 * 32)],
        delay_timer: 0,
        sound_timer: 0,
        stack: [0; 16],
        sp: 0,
    };
    cpu.initialize();

    cpu.run_opcode(0x1201); // PC should jump to 0x201

    assert_eq!(cpu.pc, 0x201);
}

#[test]
fn test_op_2nnn() {
    let mut cpu = Cpu {
        memory: [0; 4096],
        opcode: 0,
        v: [0; 16],
        i: 0,
        pc: 0,
        gfx: [0; (64 * 32)],
        delay_timer: 0,
        sound_timer: 0,
        stack: [0; 16],
        sp: 0,
    };
    cpu.initialize();

    cpu.run_opcode(0x2201);

    // sp incremented
    assert_eq!(cpu.sp, 1);

    // pc stored on stack at cpu.sp
    assert_eq!(cpu.stack[cpu.sp], 0x200);

    // pc set to nnn
    assert_eq!(cpu.pc, 0x201);
}

#[test]
fn test_op_3xkk() {
    // Skip next if Vx = kk
    let mut cpu = Cpu {
        memory: [0; 4096],
        opcode: 0,
        v: [0; 16],
        i: 0,
        pc: 0,
        gfx: [0; (64 * 32)],
        delay_timer: 0,
        sound_timer: 0,
        stack: [0; 16],
        sp: 0,
    };
    cpu.initialize();

    let mut p = cpu.pc; // Starts at 0x200
    let x: usize = 1;
    cpu.v[x] = 3 as u8;

    // After skip, cpu.pc should have moved up two opcode size
    cpu.run_opcode(0x3103);
    assert_eq!(cpu.pc, p + (OPCODE_SIZE * 2));

    // Should not skip, cpu.pc should be up one opcode size
    p = cpu.pc;
    cpu.run_opcode(0x3104); // 3 != 4
    assert_eq!(cpu.pc, p + OPCODE_SIZE);
}

#[test]
fn test_op_4xkk() {
    // Skip next if Vx != kk
    let mut cpu = Cpu {
        memory: [0; 4096],
        opcode: 0,
        v: [0; 16],
        i: 0,
        pc: 0,
        gfx: [0; (64 * 32)],
        delay_timer: 0,
        sound_timer: 0,
        stack: [0; 16],
        sp: 0,
    };
    cpu.initialize();

    let mut p = cpu.pc; // Starts at 0x200
    let x: usize = 1;
    cpu.v[x] = 3 as u8;

    // Should skip
    cpu.run_opcode(0x4101); // 3 != 1
    assert_eq!(cpu.pc, p + (OPCODE_SIZE * 2));

    // Should not skip
    p = cpu.pc;
    cpu.run_opcode(0x4103); // 3 = 1
    assert_eq!(cpu.pc, p + OPCODE_SIZE);
}

#[test]
fn test_op_5xy0() {
    // Skip next if Vx = Vy
    let mut cpu = Cpu {
        memory: [0; 4096],
        opcode: 0,
        v: [0; 16],
        i: 0,
        pc: 0,
        gfx: [0; (64 * 32)],
        delay_timer: 0,
        sound_timer: 0,
        stack: [0; 16],
        sp: 0,
    };
    cpu.initialize();

    cpu.v[0] = 1;
    cpu.v[1] = 1;

    let mut pc = cpu.pc;
    cpu.run_opcode(0x5010); // v[0] == v[1] ( should skip )
    assert_eq!(cpu.pc, pc + (OPCODE_SIZE * 2));

    pc = cpu.pc;
    cpu.run_opcode(0x5020); // v[0] != v[2] ( should not skip )
    assert_eq!(cpu.pc, pc + OPCODE_SIZE);
}

#[test]
fn test_op_6xkk() {
    // Set Vx = kk
    let mut cpu = Cpu {
        memory: [0; 4096],
        opcode: 0,
        v: [0; 16],
        i: 0,
        pc: 0,
        gfx: [0; (64 * 32)],
        delay_timer: 0,
        sound_timer: 0,
        stack: [0; 16],
        sp: 0,
    };
    cpu.initialize();

    let pc = cpu.pc;
    cpu.run_opcode(0x61F0);

    // Vx should = F0
    assert_eq!(cpu.v[1], 0xF0);
    assert_eq!(cpu.pc, pc + OPCODE_SIZE);
}

#[test]
fn test_op_7xkk() {
    let mut cpu = Cpu {
        memory: [0; 4096],
        opcode: 0,
        v: [0; 16],
        i: 0,
        pc: 0,
        gfx: [0; (64 * 32)],
        delay_timer: 0,
        sound_timer: 0,
        stack: [0; 16],
        sp: 0,
    };
    cpu.initialize();

    let mut pc = cpu.pc;
    let mut x: usize = 0;
    assert_eq!(cpu.v[x], 0x00);

    // Test add without overflow
    cpu.run_opcode(0x7001);
    assert_eq!(cpu.v[x], 0x01);
    assert_eq!(cpu.pc, pc + OPCODE_SIZE);

    // Test add with overflow on a different register
    x = 1;
    pc = cpu.pc;
    cpu.run_opcode(0x71ff);
    assert_eq!(cpu.v[x], u8::max_value());
    assert_eq!(cpu.pc, pc + OPCODE_SIZE);

    pc = cpu.pc;
    cpu.run_opcode(0x7102);
    assert_eq!(cpu.v[x], 0x01);
    assert_eq!(cpu.pc, pc + OPCODE_SIZE);
}

#[test]
fn test_op_8xy0() {
    // Puts value Vx into Vy
    let mut cpu = Cpu {
        memory: [0; 4096],
        opcode: 0,
        v: [0; 16],
        i: 0,
        pc: 0,
        gfx: [0; (64 * 32)],
        delay_timer: 0,
        sound_timer: 0,
        stack: [0; 16],
        sp: 0,
    };
    cpu.initialize();

    let p = cpu.pc;
    cpu.v[0] = 0x05;
    cpu.run_opcode(0x8010);

    assert_eq!(0x05, cpu.v[0]);
    assert_eq!(cpu.v[0], cpu.v[1]);
    assert_eq!(cpu.pc, p + OPCODE_SIZE);
}

#[test]
fn test_op_8xy1() {
    let mut cpu = Cpu {
        memory: [0; 4096],
        opcode: 0,
        v: [0; 16],
        i: 0,
        pc: 0,
        gfx: [0; (64 * 32)],
        delay_timer: 0,
        sound_timer: 0,
        stack: [0; 16],
        sp: 0,
    };
    cpu.initialize();

    // set v[0] to b0001
    cpu.v[0] = 0b0001;
    // set v[1] to b1000
    cpu.v[1] = 0b1000;

    let pc = cpu.pc;
    // Should bitwise OR v[0] and v[1]
    cpu.run_opcode(0x8011);

    assert_eq!(cpu.v[0], 0b1001);
    assert_eq!(cpu.pc, pc + OPCODE_SIZE);
}

#[test]
fn test_op_8xy2() {
    let mut cpu = Cpu {
        memory: [0; 4096],
        opcode: 0,
        v: [0; 16],
        i: 0,
        pc: 0,
        gfx: [0; (64 * 32)],
        delay_timer: 0,
        sound_timer: 0,
        stack: [0; 16],
        sp: 0,
    };
    cpu.initialize();

    let pc = cpu.pc;
    // set v[0] to b1001
    cpu.v[0] = 0b1001;
    // set v[1] to b1011
    cpu.v[1] = 0b1011;

    // Should bitwise OR v[0] and v[1]
    cpu.run_opcode(0x8012);

    assert_eq!(cpu.v[0], 0b1001);
    assert_eq!(cpu.pc, pc + OPCODE_SIZE);
}

#[test]
fn test_op_8xy3() {
    let mut cpu = Cpu {
        memory: [0; 4096],
        opcode: 0,
        v: [0; 16],
        i: 0,
        pc: 0,
        gfx: [0; (64 * 32)],
        delay_timer: 0,
        sound_timer: 0,
        stack: [0; 16],
        sp: 0,
    };
    cpu.initialize();

    let pc = cpu.pc;

    // set v[0] to b1001
    cpu.v[0] = 0b1101;
    // set v[1] to b1011
    cpu.v[1] = 0b1011;

    // Should bitwise OR v[0] and v[1]
    cpu.run_opcode(0x8013);
    assert_eq!(cpu.v[0], 0b0110);
    assert_eq!(cpu.pc, pc + OPCODE_SIZE);
}

#[test]
fn test_op_8xy4() {
    // Vx = Vx + Vy; if carry, set VF
    let mut cpu = Cpu {
        memory: [0; 4096],
        opcode: 0,
        v: [0; 16],
        i: 0,
        pc: 0,
        gfx: [0; (64 * 32)],
        delay_timer: 0,
        sound_timer: 0,
        stack: [0; 16],
        sp: 0,
    };
    cpu.initialize();
    let mut pc = cpu.pc;

    // Test with overflow
    cpu.v[0] = 0xF0;
    cpu.v[1] = 0xF0;
    cpu.run_opcode(0x8014);
    assert_eq!(cpu.v[0], 0xE0);
    assert_eq!(cpu.v[0xF], 1);
    assert_eq!(cpu.pc, pc + OPCODE_SIZE);

    // Test without overflow
    pc = cpu.pc;
    cpu.v[2] = 0x05;
    cpu.v[3] = 0x02;
    cpu.run_opcode(0x8234);
    assert_eq!(cpu.v[2], 0x07);
    assert_eq!(cpu.v[0xF], 0);
    assert_eq!(cpu.pc, pc + OPCODE_SIZE);
}

#[test]
fn test_op_8xy5() {
    // Vx = Vx - Vy; if no carry, set VF
    let mut cpu = Cpu {
        memory: [0; 4096],
        opcode: 0,
        v: [0; 16],
        i: 0,
        pc: 0,
        gfx: [0; (64 * 32)],
        delay_timer: 0,
        sound_timer: 0,
        stack: [0; 16],
        sp: 0,
    };
    cpu.initialize();
    let mut pc = cpu.pc;

    // Test with overflow
    cpu.v[0] = 0x08;
    cpu.v[1] = 0x0A;
    cpu.run_opcode(0x8015);
    assert_eq!(cpu.v[0], 0xFE);
    assert_eq!(cpu.v[0xF], 0);
    assert_eq!(cpu.pc, pc + OPCODE_SIZE);

    // Test without overflow
    pc = cpu.pc;
    cpu.v[2] = 0x05;
    cpu.v[3] = 0x02;
    cpu.run_opcode(0x8235);
    assert_eq!(cpu.v[2], 0x03);
    assert_eq!(cpu.v[0xF], 1);
    assert_eq!(cpu.pc, pc + OPCODE_SIZE);
}

#[test]
fn test_op_8x06() {
    let mut cpu = Cpu {
        memory: [0; 4096],
        opcode: 0,
        v: [0; 16],
        i: 0,
        pc: 0,
        gfx: [0; (64 * 32)],
        delay_timer: 0,
        sound_timer: 0,
        stack: [0; 16],
        sp: 0,
    };
    cpu.initialize();

    let mut pc = cpu.pc;
    cpu.v[0] = 4;
    cpu.run_opcode(0x8006); // cpu.v[0] should = 2; with v[f] = 0;
    assert_eq!(cpu.v[0], 2);
    assert_eq!(cpu.v[0xF], 0);
    assert_eq!(cpu.pc, pc + OPCODE_SIZE);

    pc = cpu.pc;
    cpu.v[4] = 5;
    cpu.run_opcode(0x8406); // cpu.v[4] should = 2; with v[f] = 1;
    assert_eq!(cpu.v[4], 2);
    assert_eq!(cpu.v[0xF], 1);
    assert_eq!(cpu.pc, pc + OPCODE_SIZE);
}
