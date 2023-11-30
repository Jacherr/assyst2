/// Arguments are passed to commands when executed in order to provide data to the commands.
pub struct Argument {
    /// The name of the argument is used in slash commands and the help command.
    pub name: String,
    /// The type of argument, to aid with command processing.
    pub r#type: ArgumentType
}

/// The inner type of the argument.
pub enum ArgumentType {
    /// A single 'word'. Words are separated by whitespaces,.
    Word,
    /// The entire collection of arguments as a string, from this point onwards.
    String,
    /// The parsed URL to a media source. This could be an image, video, audio etc.
    MediaUrl,
    /// The buffer of a media source. This could be an image, video, audio etc.
    MediaBuffer,
    /// A whole number, parsed as an i128.
    Integer,
    /// A floating-point number, parsed as a f64.
    Float,
    /// A selection between different string options.
    Choice(&'static [&'static str]),
    /// An optional argument.
    Optional(Box<ArgumentType>),
    /// An optional argument, with a default value.
    OptionalWithDefault(Box<ArgumentType>, &'static str)
}

/// An argument after parsing.
pub enum ParsedArgument {
    /// Text argument. Can be constructed from ArgumentType::Word, ArgumentType::String, ArgumentType::MediaUrl, ArgumentType::Choice, ArgumentType::OptionalWithDefault
    Text(String),
    /// Integer argument. Can be constructed from ArgumentType::Integer
    Integer(i128),
    /// Float argument. Can be constructed from ArgumentType::Float
    Float(f64),
    /// Buffer argument. Can be constructed from ArgumentType::MediaBuffer
    Buffer(Vec<u8>),
    /// No provided value for argument. Can be constructed from ArgumentType::Optional
    Empty
}