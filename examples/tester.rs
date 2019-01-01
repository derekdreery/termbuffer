use termion::cursor::Goto;

fn main() {

    println!("{} # <- Column 5 row 1", Goto(5, 1));
}

