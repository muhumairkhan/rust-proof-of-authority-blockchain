use rust_poa_chain::{Account, print_description, transfer};

fn main() {
    println!("{} node starting...", rust_poa_chain::name());

    let mut alice = Account::new(100);
    let mut bob = Account::new(0);

    print_description(&alice);
    print_description(&bob);

    match transfer(&mut alice, &mut bob, 30) {
        Ok(()) => println!("Transferred 30"),
        Err(e) => println!("Transfer failed: {:?}", e),
    }

    print_description(&alice);
    print_description(&bob);

    match transfer(&mut alice, &mut bob, 500) {
        Ok(()) => println!("Transferred 500"),
        Err(e) => println!("Transfer failed: {:?}", e),
    }
}
