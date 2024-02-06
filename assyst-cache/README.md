# assyst-cache

assyst-cache is a relatively simple crate which holds maps and sets of data which may be undesirable to lose in the event of a restart of assyst-core, but are not important enoughto necessitate a table in the database for them.

One good example of this is the list of guild IDs Assyst is in - this information is *only* sent when the Discord WebSocket gateway connects to Assyst, and so if assyst-core held this information, it would be lost and impossible to retrieve without an (expensive) restart of the gateway. As a result, this crate holds this data so after assyst-core is restarted, it is held and can still be accessed.

For more information on each cache, refer to the doc comment for the cache.