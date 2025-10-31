use gantry_cia402::driver::event::MotorEvent;
use oze_canopen::canopen::NodeId;
use std::collections::HashMap;
use tracing::trace;

use crate::{
    axis::Axis,
    event::types::{
        HomingFeedback, PositionModeFeedback, TorqueModeFeedback, VelocityModeFeedback,
    },
};

/// Collects relevant events and information from [`MotorEvents`] of all motors on a given axis and
/// determines how these are combined into a single [`GantryEvent`]
#[derive(Debug)]
pub struct EventCombiner {
    axis: Axis,
    homing_feedback: HashMap<NodeId, HomingFeedback>,
    pos_mode_feedback: HashMap<NodeId, PositionModeFeedback>,
    vel_mode_feedback: HashMap<NodeId, VelocityModeFeedback>,
    torque_mode_feedback: HashMap<NodeId, TorqueModeFeedback>,
}

impl EventCombiner {
    pub fn new_for_axis(axis: Axis, master: NodeId, slave: Option<NodeId>) -> Self {
        trace!("Constructing EventCombiner for {axis:?} - master: {master} - slave: {slave:?}");

        let mut homing_feedback = HashMap::new();
        let mut pos_mode_feedback = HashMap::new();
        let mut vel_mode_feedback = HashMap::new();
        let mut torque_mode_feedback = HashMap::new();

        homing_feedback.insert(master, HomingFeedback::default());
        pos_mode_feedback.insert(master, PositionModeFeedback::default());
        vel_mode_feedback.insert(master, VelocityModeFeedback::default());
        torque_mode_feedback.insert(master, TorqueModeFeedback::default());

        if let Some(slave_id) = slave {
            homing_feedback.insert(slave_id, HomingFeedback::default());
            pos_mode_feedback.insert(slave_id, PositionModeFeedback::default());
            vel_mode_feedback.insert(slave_id, VelocityModeFeedback::default());
            torque_mode_feedback.insert(slave_id, TorqueModeFeedback::default());
        }

        Self {
            axis,
            homing_feedback,
            pos_mode_feedback,
            vel_mode_feedback,
            torque_mode_feedback,
        }
    }

    /// Combines/merges MotorEvents of all motors connected to a single axis into one
    /// Ganty-axis uses this to merge [`MotorEvent`] into [`GantryEvent`] to allow users to listen
    /// for e.g. Homing Completed on an axis level
    pub fn update(&mut self, node: NodeId, event: MotorEvent) -> Option<MotorEvent> {
        trace!("Updating node {node} EventCombiner for event {event:?}");

        match event {
            MotorEvent::HomingFeedback {
                at_home,
                homing_completed,
                homing_error,
            } => {
                trace!("Updating HomingFeedback for node {node} with event {event:?}");

                // Aggregate this new piece of feedback
                self.homing_feedback.insert(
                    node,
                    HomingFeedback {
                        at_home,
                        homing_completed,
                        homing_error,
                    },
                );

                // Combine it into a single level feedback
                let combined = self.combine_homing_feedback();

                trace!("Combined HomingFeedback for node {node} into {combined:?}");

                Some(MotorEvent::HomingFeedback {
                    at_home: combined.at_home,
                    homing_completed: combined.homing_completed,
                    homing_error: combined.homing_error,
                })
            }

            MotorEvent::PositionModeFeedback {
                target_reached,
                limit_exceeded,
                setpoint_acknowlegded,
                following_error,
            } => {
                // Aggregate this new piece of feedback
                self.pos_mode_feedback.insert(
                    node,
                    PositionModeFeedback {
                        target_reached,
                        limit_exceeded,
                        setpoint_acknowlegded,
                        following_error,
                    },
                );

                // Combine it into a single Axis level feedback
                let combined = self.combine_position_feedback();

                // Send that out as gantry event so users can listen for relevant progress on axis level
                Some(MotorEvent::PositionModeFeedback {
                    target_reached: combined.target_reached,
                    limit_exceeded: combined.limit_exceeded,
                    setpoint_acknowlegded: combined.setpoint_acknowlegded,
                    following_error: combined.following_error,
                })
            }

            MotorEvent::VelocityModeFeedback {
                speed_is_zero,
                deviation_error,
            } => {
                // Aggregate this new piece of feedback
                self.vel_mode_feedback.insert(
                    node,
                    VelocityModeFeedback {
                        speed_is_zero,
                        deviation_error,
                    },
                );

                // Combine it into a single Axis level feedback
                let combined = self.combine_velocity_feedback();

                // Send that out as gantry event so users can listen for relevant progress on axis level
                Some(MotorEvent::VelocityModeFeedback {
                    speed_is_zero: combined.speed_is_zero,
                    deviation_error: combined.deviation_error,
                })
            }

            MotorEvent::TorqueModeFeedback {
                axis_braked,
                setpoint_reached,
                limit_exceeded,
            } => {
                // Aggregate this new piece of feedback
                self.torque_mode_feedback.insert(
                    node,
                    TorqueModeFeedback {
                        axis_braked,
                        setpoint_reached,
                        limit_exceeded,
                    },
                );

                // Combine it into a single Axis level feedback
                let combined = self.combine_torque_feedback();

                // Send that out as gantry event so users can listen for relevant progress on axis level
                Some(MotorEvent::TorqueModeFeedback {
                    axis_braked: combined.axis_braked,
                    setpoint_reached: combined.setpoint_reached,
                    limit_exceeded: combined.limit_exceeded,
                })
            }

            _ => None,
        }
    }

    /// Combines all aggregated homing feedback into a single piece of information
    pub fn combine_homing_feedback(&self) -> HomingFeedback {
        trace!("Combining homing feedback for {self:?}");

        HomingFeedback {
            at_home: self.homing_feedback.values().all(|f| f.at_home),
            homing_completed: self.homing_feedback.values().all(|f| f.homing_completed),
            homing_error: self.homing_feedback.values().any(|f| f.homing_error),
        }
    }

    /// Combines aggregated events into a single piece of feedback to be sent as GantryEvent
    pub fn combine_position_feedback(&self) -> PositionModeFeedback {
        PositionModeFeedback {
            target_reached: self.pos_mode_feedback.values().all(|f| f.target_reached),
            limit_exceeded: self.pos_mode_feedback.values().any(|f| f.limit_exceeded),
            setpoint_acknowlegded: self
                .pos_mode_feedback
                .values()
                .all(|f| f.setpoint_acknowlegded),
            following_error: self.pos_mode_feedback.values().any(|f| f.following_error),
        }
    }

    /// Combines all aggregated velocity feedback into a single piece of information
    pub fn combine_velocity_feedback(&self) -> VelocityModeFeedback {
        VelocityModeFeedback {
            speed_is_zero: self.vel_mode_feedback.values().all(|f| f.speed_is_zero),
            deviation_error: self.vel_mode_feedback.values().any(|f| f.deviation_error),
        }
    }

    /// Combines all aggregated torque feedback into a single piece of information
    pub fn combine_torque_feedback(&self) -> TorqueModeFeedback {
        TorqueModeFeedback {
            axis_braked: self.torque_mode_feedback.values().any(|f| f.axis_braked),
            setpoint_reached: self
                .torque_mode_feedback
                .values()
                .all(|f| f.setpoint_reached),
            limit_exceeded: self.torque_mode_feedback.values().any(|f| f.limit_exceeded),
        }
    }
}
