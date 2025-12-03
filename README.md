# CHIP-8 Emulator

<div align="center">

  ![alt text](image.png)
  
</div>

## 🎮 About

A **CHIP-8 virtual machine emulator** written in Rust, created to practice Rust. This project implements the classic 1970s interpreted programming language that powered many early microcomputer games.

## 🕹️ Opcode Highlights

This emulator implements the complete instruction set, including the **historically interesting `0x8XYE` opcode**:

```asm
SHL Vx {, Vy}  ; Shift Vx left by 1, MSB → VF
```

In Super CHIP-8, this opcode was redefined from the original CHIP-8's simple register copy to a bitwise shift operation. Here we have both the original and the new one, where Vy is read, shifted, and stored in Vx, with the most significant bit preserved in the carry flag VF. 

This instruction was required to run [8CE Attourny](https://johnearnest.github.io/chip8Archive/play.html?p=8ceattourny_d1).

## 🎯 Modern CHIP-8 Games

The CHIP-8 scene is alive and well! Check out these amazing modern creations at the [CHIP-8 Archive](https://johnearnest.github.io/chip8Archive/#Octojam1). Note that some of those require a few other instructions that compose whats called the **Super Chip-8**.