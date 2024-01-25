/// Initial Discord message processing.
/// Checks the validity of the message before performing any kind of parsing.
///
/// This includes:
/// - Checking if the author is globally blacklisted from running commands,
/// - Checking if the author is blacklisted in the guild from running commands,
/// - Checking that the message is not sent by a bot or a webhook,
/// - Checking that the message starts with the correct prefix for the context, and returning any
///   identified prefix.
pub fn preprocess() {
    
}
