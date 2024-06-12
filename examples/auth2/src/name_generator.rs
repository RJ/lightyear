pub fn sanitise_name(id: u64, name: String) -> String {
    if name.trim().is_empty() || name.len() > 15 {
        hashed_name(id)
    } else {
        name.trim().to_string()
    }
}

pub fn hashed_name(id: u64) -> String {
    let idx = (id % NAMES.len() as u64) as usize;
    NAMES[idx].to_string()
}

const NAMES: [&str; 50] = [
    "dog",
    "cat",
    "bird",
    "fish",
    "hamster",
    "rabbit",
    "turtle",
    "guinea pig",
    "lizard",
    "snake",
    "frog",
    "horse",
    "chicken",
    "duck",
    "goat",
    "sheep",
    "cow",
    "pig",
    "parrot",
    "canary",
    "budgie",
    "mouse",
    "rat",
    "gerbil",
    "ferret",
    "hedgehog",
    "chinchilla",
    "tarantula",
    "scorpion",
    "gecko",
    "iguana",
    "newt",
    "salamander",
    "chameleon",
    "pony",
    "alpaca",
    "llama",
    "donkey",
    "mule",
    "pigeon",
    "dove",
    "quail",
    "turkey",
    "peacock",
    "goose",
    "swan",
    "bison",
    "buffalo",
    "elk",
    "deer",
];
