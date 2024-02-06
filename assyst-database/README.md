# assyst-database

Main database handling crate for Assyst. Handles all interactions with the database, in this case [PostgreSQL](https://www.postgresql.org/), and also contains abstrated caching logic for frequently accessed areas of the database (for example, prefixes on message commands).

This crate is split into multiple separate structs, each one responsible for a table. In addition, the `impl` of each struct contains reading and writing methods for easy interfacing with that table, allowing the storage and retieval of data without needing to worry about the SQL queries involved. The function of each struct is documented using doc comments on the struct itself.