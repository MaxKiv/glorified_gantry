pub mod common;

use tracing::*;

#[cfg(test)]
mod tests {
    
    use gantry_cia402::driver::{identifier::Cia402Identifier, startup::params::TEST_PARAMS};
    use gantry_demo::config::{
        DEMO_SETUP_DEVICE_NAME, TEST_SETUP_MOTOR_TYPE,
        TEST_SETUP_PROFILE_NUMBER,
    };
    

    use gantry_axis::{
        axis::{Axis, AxisConfig},
        cfg::GantryConfig,
        gantry::Gantry,
        setpoint::translator::scaling::DeviceScaling,
    };

    use super::*;

    pub const BAD_X_CONFIG: Option<AxisConfig> = Some(AxisConfig {
        axis: Axis::X,
        master: Cia402Identifier {
            node_id: 1,
            device_profile_number: TEST_SETUP_PROFILE_NUMBER,
            motor_type: TEST_SETUP_MOTOR_TYPE,
            device_name: DEMO_SETUP_DEVICE_NAME, // NOTE: demo name in test setup identifier
        },
        slave: None,
        params: TEST_PARAMS,
        scaling: DeviceScaling::test_setup(),
    });

    pub const BAD_CONFIG: GantryConfig = GantryConfig {
        x: BAD_X_CONFIG,
        y: None,
        z: None,
    };

    #[tokio::test]
    async fn startup_failure_test() -> anyhow::Result<()> {
        gantry_demo::setup_tracing();

        info!("Starting can interface");
        let (canopen, _) = oze_canopen::canopen::start(String::from("can0"), Some(1_000_000));

        let gantry_init_result = Gantry::start(canopen, BAD_CONFIG).await;

        assert!(
            gantry_init_result.is_err(),
            "Initialising the Gantry with bad config should error, but didn't"
        );

        Ok(())
    }
}
