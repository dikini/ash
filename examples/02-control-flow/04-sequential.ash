// Sequential - Ordered workflow composition
//
// This workflow demonstrates composing operations sequentially, 
// where each step depends on or follows from the previous step.

capability database_connect : act(database: String) returns Connection
capability database_query : act(conn: Connection, query: Query) returns Results
capability prepare_query : analyze(q: String) returns Query

workflow main {
    // Execute steps in sequence, passing results forward
    
    // Step 1: Initialize configuration
    let config = {
        database: "production",
        timeout: 30,
        retries: 3
    }
    
    // Step 2: Validate configuration
    if config.timeout <= 0 {
        ret { error: "Invalid timeout" }
    }
    
    // Step 3: Connect to database
    act database_connect(config.database) as conn
    
    // Step 4: Prepare query
    orient {
        prepare_query("SELECT * FROM events")
    } as query
    
    // Step 5: Execute query
    act database_query(conn, query) as results
    
    // Step 6: Process and return
    orient {
        {
            count: length(results),
            first: head(results),
            last: last(results)
        }
    } as summary
    
    ret summary
}
