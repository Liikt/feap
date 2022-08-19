use std::collections::HashMap;

#[macro_export]
macro_rules! start_timer {
    ($timer:expr, $feature:expr) => {{
        #[cfg(feature = "introspection")]
        $timer.start_timer($feature);
    }};
}

#[macro_export]
macro_rules! mark_timer {
    ($timer:expr, $feature:expr) => {{
        #[cfg(feature = "introspection")]
        $timer.mark_timer($feature);
    }};
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TimerHook {
    RemoveChildHook,
    MergingHook,
    InnerMergingLoop,
    UpdatingHook,
    FastRootListInsert,
    SlowRootListInsert,
}

#[derive(Clone)]
pub struct Timer {
    name: String,
    times: HashMap<TimerHook, (u128, u64)>,
    timers: HashMap<TimerHook, Option<u64>>,
}

impl Timer {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_owned(),
            times: HashMap::new(),
            timers: HashMap::new()
        }
    }

    #[inline]
    pub fn start_timer(&mut self, feature: TimerHook) {
        self.timers.insert(feature.into(), 
            unsafe { Some(std::arch::x86_64::_rdtsc()) });
    }

    #[inline]
    pub fn mark_timer(&mut self, feature: TimerHook) {
        let stop = unsafe { std::arch::x86_64::_rdtsc() };
        match self.timers.get(&feature.into()) {
            // Feature exists and has a starting time
            Some(Some(start)) => {
                let (cur_sum, times_meassured) = self.times.entry(feature.into())
                    .or_insert((0, 0));
                *cur_sum += (stop - start) as u128;
                *times_meassured += 1;
                self.timers.insert(feature.into(), None);
            },
            // Feature exist, but timer hasn't started
            Some(None) => {},
            // Feature does not exist
            None => {
                panic!("Tried to mark a non existing feature {:?}", feature);
            }
        }
    }
}

impl std::fmt::Debug for Timer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Name: {}\n", self.name)?;
        for (&k, &(v, t)) in self.times.iter() {
            write!(f, "{:?}: {}\n", k, v/(t as u128))?;
        }
        Ok(())
    }
}