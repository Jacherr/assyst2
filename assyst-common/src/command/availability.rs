/// Defines who can use a command in a server.
pub enum Availability {
    /// Anyone can use this command, subject to blacklisting and whitelisting configuration.
    Public,
    /// Server managers (those with the 'manage server' permission) can use this command.
    ServerManagers,
    /// Only developers, as configured, can use this command.
    Dev
}