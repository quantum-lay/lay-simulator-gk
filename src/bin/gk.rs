use lay::{Layer, OpsVec};
use lay_simulator_gk::{GottesmanKnillSimulator, BitArray};

fn main() {
    let mut cli = GottesmanKnillSimulator::from_seed(2, 0);
    let mut ops = OpsVec::<GottesmanKnillSimulator<_>>::new();
    ops.x(0);
    ops.cx(0, 1);
    cli.send(ops.as_ref());
    cli.dump_print();
    ops.clear();
    ops.measure(0, 0);
    cli.send(ops.as_ref());
    cli.dump_print();
    ops.clear();
    ops.measure(1, 1);
    cli.send(ops.as_ref());
    cli.dump_print();
    let mut result = BitArray::zeros(0);
    cli.receive(&mut result);
    println!("result: {:?}", result);
}
