// Sequential - Ordered workflow composition
//
// This workflow demonstrates sequential execution with the seq operator.

workflow main {
    // Execute steps in sequence, passing results forward
    seq {
        // Step 1: Initialize
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
        observe database_connect(config.database) as conn
        
        // Step 4: Execute query
        orient {
            prepare_query("SELECT * FROM events")
        } as query
        
        // Step 5: Fetch results
        observe database_query(conn, query) as results
        
        // Step 6: Process and return
        orient {
            {
                count: length(results),
                first: head(results),
                last: last(results)
            }
        } as summary
    }
    
    ret summary
}
