// Sequential - Ordered workflow composition
//
// This workflow demonstrates composing operations sequentially, 
// where each step depends on or follows from the previous step.

capability data_source {
    effect: read,
    params: [source: String],
    returns: Data
}

workflow main {
    // Fetch data from multiple sources sequentially
    // Each fetch builds on the previous step's context
    
    // Step 1: Fetch user data
    let userData = observe data_source("users")
    orient {
        validate_user(userData)
    }
    
    // Step 2: Fetch order data (after users are validated)
    let orderData = observe data_source("orders")
    orient {
        summarize_orders(orderData)
    }
    
    // Step 3: Fetch inventory data (after orders are summarized)
    let inventoryData = observe data_source("inventory")
    orient {
        check_stock(inventoryData)
    }
    
    // After all steps complete, combine results
    orient {
        let report = {
            users: userData,
            orders: orderData,
            inventory: inventoryData,
            timestamp: now()
        }
    } as combinedReport
    
    decide {
        if combinedReport.inventory.low_stock {
            action "reorder_supplies"
        }
    }
    
    ret combinedReport
}
