-- Simple Temperature Monitoring Workflow
-- Demonstrates: capabilities, policies, OODA pattern, control flow

-- Define capabilities for the workflow
capability read_sensor : observe(sensor_id: String) returns Reading
capability analyze_reading : analyze(reading: Reading) returns Analysis
capability send_alert : act(severity: String, message: String) where notified
capability log_status : write(entry: LogEntry)

-- Define a policy for high temperature alerts
policy temperature_alert:
  when temp > 90
  then require_approval(role: supervisor)
  else permit

-- Define a policy for normal operations
policy standard_operation:
  when temp >= 0 and temp <= 100
  then permit
  else escalate

workflow temperature_monitor {
  -- OBSERVE: Read from temperature sensor
  observe read_sensor with sensor_id: "temp_001" as reading;
  
  -- ORIENT: Analyze the reading
  orient { analyze_reading(reading) } as analysis;
  
  -- Variable binding for status
  let status = if reading.temp > 80 {
    "critical"
  } else if reading.temp > 60 {
    "warning"
  } else {
    "normal"
  };
  
  -- DECIDE: Apply policy check
  decide { reading.temp } under temperature_alert then {
    -- ACT: Execute based on decision
    act send_alert(
      severity: status,
      message: "Temperature " + status + ": " + reading.temp
    ) where notified;
    
    act log_status({
      sensor: "temp_001",
      temp: reading.temp,
      status: status,
      timestamp: now()
    });
    
    ret { alert_sent: true, status: status }
  } else {
    -- Parallel logging for normal operations
    par {
      act log_status({
        sensor: "temp_001",
        temp: reading.temp,
        status: status,
        timestamp: now()
      });
      
      orient { summarize_trend(analysis) }
    };
    
    ret { alert_sent: false, status: status }
  }
}
