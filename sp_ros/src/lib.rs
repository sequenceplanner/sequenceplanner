#[cfg(not(feature = "ros"))]
#[macro_use]
mod ros {
    use sp_domain::*;

    /// empty when no ros support compiled in.
    pub struct RosComm {
    }

    impl RosComm {
        pub async fn new(
            _state_from_runner: tokio::sync::watch::Receiver<SPState>,
            _state_to_runner: tokio::sync::mpsc::Sender<SPState>,
            _initial_model: &impl sp_model::Resource,
        ) -> Result<RosComm, SPError> {
            panic!("You need ros to run ros. Enable the ros feature")
        }

        pub fn abort(&self)   {
            panic!("You need ros to run ros. Enable the ros feature")
        }
        pub async fn abort_and_await(&mut self) -> Result<(), SPError>  {
            panic!("You need ros to run ros. Enable the ros feature")
        }
    }


    pub fn log(msg: &str, file: &str, line: u32, severity: u32) {
        println!("{}:{}:[{}] - {}", file, line, severity, msg);
    }

    pub fn log_debug(msg: &str, file: &str, line: u32) {
        log(msg, file, line, 1);
    }
    pub fn log_info(msg: &str, file: &str, line: u32) {
        log(msg, file, line, 2);
    }
    pub fn log_warn(msg: &str, file: &str, line: u32) {
        log(msg, file, line, 3);
    }
    pub fn log_error(msg: &str, file: &str, line: u32) {
        log(msg, file, line, 4);
    }
    pub fn log_fatal(msg: &str, file: &str, line: u32) {
        log(msg, file, line, 5);
    }

    #[macro_export]
    macro_rules! log_debug {
        ($($args:tt)*) => {{
            $crate::log(&std::fmt::format(format_args!($($args)*)), file!(), line!(), 1);
        }}
    }

    #[macro_export]
    macro_rules! log_info {
        ($($args:tt)*) => {{
            $crate::log(&std::fmt::format(format_args!($($args)*)), file!(), line!(), 2);
        }}
    }

    #[macro_export]
    macro_rules! log_warn {
        ($($args:tt)*) => {{
            $crate::log(&std::fmt::format(format_args!($($args)*)), file!(), line!(), 3);
        }}
    }

    #[macro_export]
    macro_rules! log_error {
        ($($args:tt)*) => {{
            $crate::log(&std::fmt::format(format_args!($($args)*)), file!(), line!(), 4);
        }}
    }

    #[macro_export]
    macro_rules! log_fatal {
        ($($args:tt)*) => {{
            $crate::log(&std::fmt::format(format_args!($($args)*)), file!(), line!(), 5);
        }}
    }


}

#[cfg(feature = "ros")]
#[macro_use]
mod ros {
    use std::sync::{Arc, Mutex};
    use sp_domain::*;
    use sp_model::*;
    // use crate::state_service::SPStateService;

    use super::resource_comm::*;

    pub const SP_NODE_NAME: &str = "sp";


    pub fn log_debug(msg: &str, file: &str, line: u32) {
        r2r::log(msg, SP_NODE_NAME, file, line, r2r::LogSeverity::Debug);
    }

    #[macro_export]
    macro_rules! log_debug {
        ($($args:tt)*) => {{
            $crate::log_debug(&std::fmt::format(format_args!($($args)*)), file!(), line!());
        }}
    }

    pub fn log_info(msg: &str, file: &str, line: u32) {
        println!("{}:{} - {}", file, line, msg);
        r2r::log(msg, SP_NODE_NAME, file, line, r2r::LogSeverity::Info);
    }

    #[macro_export]
    macro_rules! log_info {
        ($($args:tt)*) => {{
            $crate::log_info(&std::fmt::format(format_args!($($args)*)), file!(), line!());
        }}
    }

    pub fn log_warn(msg: &str, file: &str, line: u32) {
        r2r::log(msg, SP_NODE_NAME, file, line, r2r::LogSeverity::Warn);
    }

    #[macro_export]
    macro_rules! log_warn {
        ($($args:tt)*) => {{
            $crate::log_warn(&std::fmt::format(format_args!($($args)*)), file!(), line!());
        }}
    }

    pub fn log_error(msg: &str, file: &str, line: u32) {
        r2r::log(msg, SP_NODE_NAME, file, line, r2r::LogSeverity::Error);
    }

    #[macro_export]
    macro_rules! log_error {
        ($($args:tt)*) => {{
            $crate::log_error(&std::fmt::format(format_args!($($args)*)), file!(), line!());
        }}
    }

    pub fn log_fatal(msg: &str, file: &str, line: u32) {
        r2r::log(msg, SP_NODE_NAME, file, line, r2r::LogSeverity::Fatal);
    }

    #[macro_export]
    macro_rules! log_fatal {
        ($($args:tt)*) => {{
            $crate::log_fatal(&std::fmt::format(format_args!($($args)*)), file!(), line!());
        }}
    }

    pub struct RosComm {
        arc_node: Arc<Mutex<r2r::Node>>,
        spin_handle: tokio::task::JoinHandle<()>,
        resources_handle: tokio::task::JoinHandle<()>,
        // sp_state: SPStateService,
//        sp_model: SPModelService,
        resources: Arc<Mutex<Vec<ResourceComm>>>,
    }

    impl RosComm {
        pub async fn new(
            state_from_runner: tokio::sync::watch::Receiver<SPState>,
            state_to_runner: tokio::sync::mpsc::Sender<SPState>,
            initial_model: impl Resource,
        ) -> Result<RosComm, SPError> {
            let ctx = r2r::Context::create().map_err(SPError::from_any)?;
            let node = r2r::Node::create(ctx, SP_NODE_NAME, "").map_err(SPError::from_any)?;
            let arc_node = Arc::new(Mutex::new(node));
            let resources =  Arc::new(Mutex::new(vec!()));

            // let sp_state = SPStateService::new(
            //     arc_node.clone(),
            //     state_from_runner.clone(),
            //     state_to_runner.clone()
            // ).await?;

//            let model_watcher = sp_model.model_watcher();
            let resources_handle = RosComm::launch_resources(
                arc_node.clone(),
                resources.clone(),
                state_from_runner.clone(),
                state_to_runner.clone(),
                initial_model,
            ).await;

            let task_arc_node = arc_node.clone();
            let spin_handle = tokio::task::spawn_blocking( move || {
                std::thread::sleep(std::time::Duration::from_millis(100));
                loop {
                    {
                        let mut node = task_arc_node.lock().unwrap();
                        node.spin_once(std::time::Duration::from_millis(10));
                    }
                    std::thread::sleep(std::time::Duration::from_millis(10));

                }
            });

            let rc = RosComm {
                arc_node,
                spin_handle,
                resources_handle,
//                sp_state,
                resources,
//                model_watcher: model_watcher.clone(),
            };


            Ok(rc)

        }

        pub fn abort(&self)   {
            // self.sp_model.abort();
            // self.sp_state.abort();
            self.spin_handle.abort();
            let rs = self.resources.lock().unwrap();
            rs.iter().for_each(|r| r.abort());
        }
        pub async fn abort_and_await(&mut self) -> Result<(), SPError>  {
            // self.sp_model.abort_and_await().await?;
            // self.sp_state.abort_and_await().await?;
            self.spin_handle.abort();
            let mut rs = vec!();
            std::mem::swap(&mut *self.resources.lock().unwrap(), &mut rs);

            for mut r in rs.into_iter() {
                r.abort_and_await().await?;
            }
            Ok(())
        }

        async fn launch_resources(
            arc_node: Arc<Mutex<r2r::Node>>,
            resources: Arc<Mutex<Vec<ResourceComm>>>,
            state_from_runner: tokio::sync::watch::Receiver<SPState>,
            state_to_runner: tokio::sync::mpsc::Sender<SPState>,
            model: impl Resource,
        ) -> tokio::task::JoinHandle<()> {

            tokio::spawn(async move {
                let arc_node = arc_node.clone();
                loop {
                    // new_res.iter().for_each(|r| println!("XXX RESOURCES: {:?}", r.path()));
                    // for r in new_res {
                    //     let rc =  ResourceComm::new(
                    //         arc_node.clone(),
                    //         r.clone(),
                    //         state_from_runner.clone(),
                    //         state_to_runner.clone()
                    //     );
                    //     match rc.await {
                    //         Ok(comm) => {
                    //             let mut res = resources.lock().unwrap();
                    //             res.push(comm);
                    //         },
                    //         Err(e) => {
                    //             log_warn!("The comm for resource {} couldn't be created: {}", r.path(), e);
                    //         }
                    //     }

                    // }

                }
            })
        }


    }





}

pub use ros::*;

#[cfg(feature = "ros")]
mod resource_comm;

// TODO: add back
// #[cfg(feature = "ros")]
// mod state_service;
