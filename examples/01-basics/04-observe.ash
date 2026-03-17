// Observe - The OODA pattern
//
// This workflow demonstrates the Observe-Orient-Decide-Act (OODA) pattern,
// which is fundamental to Ash workflows.

// Define a capability for reading sensor data
capability sensor_reader {
    effect: observe,
    params: [sensor_id: String],
    returns: SensorData
}

// Define a policy for data validation
policy valid_reading {
    condition: data.temperature > -50 && data.temperature < 150,
    decision: permit
}

workflow main {
    // OBSERVE: Read data from a capability
    observe sensor_reader("temp_sensor_01") as reading
    
    // ORIENT: Analyze the data
    orient {
        let status = if reading.temperature > 80 {
            "critical"
        } else if reading.temperature > 60 {
            "warning"
        } else {
            "normal"
        }
    } as analysis
    
    // DECIDE: Apply policy
    decide {
        if analysis.status == "critical" {
            action "alert_ops"
        } else {
            action "log_reading"
        }
    }
    
    // ACT: Execute the decided action
    act log_reading with guard always
    
    ret {
        reading: reading,
        analysis: analysis
    }
}
