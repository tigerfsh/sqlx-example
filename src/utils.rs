use rand::{Rng, distributions::Alphanumeric, thread_rng};
use rand::seq::SliceRandom;

pub fn generate_random_username() -> String {
    let mut rng = thread_rng();
    let username: String = (&mut rng)
        .sample_iter(Alphanumeric)
        .filter(|c| c.is_ascii_alphabetic())
        .map(char::from)
        .take(10)
        .collect();
    username
}

pub fn generate_random_email() -> String {
    let username = generate_random_username().to_lowercase();
    let domains = ["example.com", "test.com", "mail.com", "demo.org"];

    let mut rng = thread_rng();
    let domain = domains.choose(&mut rng).unwrap_or(&"example.com");
    format!("{}@{}", username, domain)
}