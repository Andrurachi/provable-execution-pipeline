fn main() {
    // Tells Cargo to compile the guest folder into a zkVM-compatible ELF file
    sp1_build::build_program("../guest");
}
