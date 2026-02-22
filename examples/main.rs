use std::env;
use std::fs;
use std::process;

use mc6809_core::{Bus, Cpu};

/// Simple 64KB flat RAM bus for testing.
struct FlatBus {
    mem: Box<[u8; 65536]>,
}

impl FlatBus {
    fn new() -> Self {
        Self {
            mem: Box::new([0u8; 65536]),
        }
    }

    fn load(&mut self, data: &[u8], base: u16) {
        let start = base as usize;
        let end = start + data.len();
        if end > 65536 {
            eprintln!("Error: data exceeds 64KB address space");
            process::exit(1);
        }
        self.mem[start..end].copy_from_slice(data);
    }

    /// Set the reset vector to point to the given address.
    fn set_reset_vector(&mut self, addr: u16) {
        self.mem[0xFFFE] = (addr >> 8) as u8;
        self.mem[0xFFFF] = addr as u8;
    }
}

impl Bus for FlatBus {
    fn read(&self, addr: u16) -> u8 {
        self.mem[addr as usize]
    }

    fn write(&mut self, addr: u16, val: u8) {
        self.mem[addr as usize] = val;
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
        eprintln!(
            "Usage: {} <binary-file> <load-address-hex> [--trace] [--max-cycles N]",
            args[0]
        );
        eprintln!();
        eprintln!("  Loads a raw binary at the specified address, sets the reset vector,");
        eprintln!("  and runs the 6809 CPU until it halts or exceeds the cycle limit.");
        eprintln!();
        eprintln!("Options:");
        eprintln!("  --trace          Print register state after each instruction");
        eprintln!("  --max-cycles N   Stop after N cycles (default: 1000000)");
        process::exit(1);
    }

    let filename = &args[1];
    let load_addr = u16::from_str_radix(&args[2], 16).unwrap_or_else(|_| {
        eprintln!("Error: invalid hex address '{}'", args[2]);
        process::exit(1);
    });

    let mut trace = false;
    let mut max_cycles: u64 = 1_000;

    let mut i = 3;
    while i < args.len() {
        match args[i].as_str() {
            "--trace" => trace = true,
            "--max-cycles" => {
                i += 1;
                max_cycles = args.get(i).and_then(|s| s.parse().ok()).unwrap_or_else(|| {
                    eprintln!("Error: --max-cycles requires a numeric argument");
                    process::exit(1);
                });
            }
            other => {
                eprintln!("Unknown option: {}", other);
                process::exit(1);
            }
        }
        i += 1;
    }

    let data = fs::read(filename).unwrap_or_else(|e| {
        eprintln!("Error reading '{}': {}", filename, e);
        process::exit(1);
    });

    let mut bus = FlatBus::new();
    bus.load(&data, load_addr);
    bus.set_reset_vector(load_addr);

    let mut cpu = Cpu::new();
    cpu.reset(&mut bus);

    println!(
        "Loaded {} bytes at {:04X}, reset vector → {:04X}",
        data.len(),
        load_addr,
        load_addr
    );
    println!("Initial state: {:?}", cpu);
    println!();

    while cpu.cycles < max_cycles && !cpu.halted {
        if trace {
            print!("{:?}  ", cpu);
        }
        let cyc = cpu.step(&mut bus);
        if trace {
            println!("({} cycles)", cyc);
        }
    }

    println!();
    if cpu.halted {
        println!("CPU halted after {} cycles", cpu.cycles);
    } else {
        println!("Cycle limit ({}) reached", max_cycles);
    }
    println!("Final state: {:?}", cpu);
}
