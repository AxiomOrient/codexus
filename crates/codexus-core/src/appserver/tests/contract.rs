use super::*;

#[test]
fn bridge_method_constants_are_stable() {
    assert_eq!(methods::INITIALIZE, "initialize");
    assert_eq!(methods::THREAD_START, "thread/start");
    assert_eq!(methods::TURN_INTERRUPT, "turn/interrupt");
    assert_eq!(methods::TURN_STEER, "turn/steer");
    assert_eq!(methods::SKILLS_LIST, "skills/list");
    assert_eq!(methods::SKILLS_CHANGED, "skills/changed");
    assert_eq!(methods::FS_WATCH, "fs/watch");
    assert_eq!(methods::FS_UNWATCH, "fs/unwatch");
    assert_eq!(methods::FS_CHANGED, "fs/changed");
    assert_eq!(
        methods::EXPERIMENTAL_FEATURE_ENABLEMENT_SET,
        "experimentalFeature/enablement/set"
    );
    assert_eq!(methods::COMMAND_EXEC, "command/exec");
    assert_eq!(
        methods::COMMAND_EXEC_OUTPUT_DELTA,
        "command/exec/outputDelta"
    );
}

#[test]
fn bridge_surface_keeps_raw_escape_hatch_available() {
    assert_eq!(methods::THREAD_START, "thread/start");
    assert_eq!(methods::COMMAND_EXEC, "command/exec");
    assert_eq!(methods::TURN_START, "turn/start");
}
