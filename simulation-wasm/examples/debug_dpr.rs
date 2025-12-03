use simulation_wasm::dice;

fn main() {
    let formulas = vec![
        "2d6 + 5[STR] + 2[WEAPON]",
        "1d6 + 5[DEX] + 10[SS]",
        "1d6+15",
        "11"
    ];

    for f in formulas {
        let avg = dice::parse_average(f);
        println!("Formula: '{}' -> Average: {}", f, avg);
    }
}
