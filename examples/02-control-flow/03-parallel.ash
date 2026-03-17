// Parallel - Concurrent workflow execution
//
// This workflow demonstrates executing branches in parallel.

capability data_source {
    effect: read,
    params: [source: String],
    returns: Data
}

workflow main {
    // Fetch data from multiple sources in parallel
    par {
        // Branch 1: Fetch user data
        let userData = observe data_source("users")
        orient {
            validate_user(userData)
        }
        
        // Branch 2: Fetch order data
        let orderData = observe data_source("orders")
        orient {
            summarize_orders(orderData)
        }
        
        // Branch 3: Fetch inventory data
        let inventoryData = observe data_source("inventory")
        orient {
            check_stock(inventoryData)
        }
    }
    
    // After all branches complete, combine results
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
