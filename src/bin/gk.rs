use lay::Operations;
use lay::gates::CliffordGate;
use lay_simulator_gk::GottesmanKnillSimulator;

fn main() {
    let mut cli = GottesmanKnillSimulator::from_seed(2, 0);
    cli.x(0);
    cli.cx(0, 1);
    cli.dump_print();
    cli.measure(0, 0);
    cli.dump_print();
    cli.measure(1, 1);
    cli.dump_print();
}
