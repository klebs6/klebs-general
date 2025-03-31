// ---------------- [ File: agent-coordinate/src/agent_coordinate.rs ]
crate::ix!();

#[derive(Clone, Debug, Getters, Builder)]
#[builder(setter(into, strip_option), default)]
#[getset(get = "pub")]
pub struct AgentCoordinate {
    location: Option<String>,
    goal:     Option<String>,
}

impl Default for AgentCoordinate {
    fn default() -> Self {
        info!("Creating default AgentCoordinate (nowhere_with_no_goal).");
        Self::nowhere_with_no_goal()
    }
}

impl AgentCoordinate {

    pub fn here_without_a_goal(location: &str) -> Self {
        info!("Creating AgentCoordinate with location='{location}' and no goal.");
        Self {
            location: Some(location.to_string()),
            goal:     None,
        }
    }

    pub fn here_with_a_goal(location: &str, goal: &str) -> Self {
        info!("Creating AgentCoordinate with location='{location}' and goal='{goal}'.");
        Self {
            location: Some(location.to_string()),
            goal:     Some(goal.to_string()),
        }
    }

    pub fn nowhere_with_no_goal() -> Self {
        info!("Creating AgentCoordinate with no location and no goal.");
        Self {
            location: None,
            goal:     None,
        }
    }

    pub fn nowhere_with_a_goal(goal: &str) -> Self {
        info!("Creating AgentCoordinate with no location and goal='{goal}'.");
        Self {
            location: None,
            goal:     Some(goal.to_string()),
        }
    }

    pub fn has_location(&self) -> bool {
        let has_loc = self.location.is_some();
        debug!("Checking if AgentCoordinate has a location: {has_loc}");
        has_loc
    }

    pub fn has_goal(&self) -> bool {
        let has_g = self.goal.is_some();
        debug!("Checking if AgentCoordinate has a goal: {has_g}");
        has_g
    }

    pub fn is_fully_specified(&self) -> bool {
        let fully = self.has_location() && self.has_goal();
        trace!("Checking if AgentCoordinate is fully specified: {fully}");
        fully
    }

    pub fn set_location(&mut self, loc: Option<&str>) {
        if self.location.is_some() {
            warn!("Overwriting existing location with '{loc:?}'");
        }
        self.location = loc.map(|x| x.to_string());
    }

    pub fn set_goal(&mut self, g: Option<&str>) {
        if self.goal.is_some() {
            warn!("Overwriting existing goal with '{g:?}'");
        }
        self.goal = g.map(|x| x.to_string());
    }
}

impl std::fmt::Display for AgentCoordinate {
    fn fmt(&self, out: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        debug!("Formatting AgentCoordinate for display.");

        if self.location.is_none() && self.goal.is_none() {
            debug!("No location and no goal specified.");
            return write!(
                out,
                "Neither your goal nor your location are specified by the user."
            );
        }

        let mut x = String::new();

        if let Some(place) = &self.location {
            debug!("Appending location='{place}' to display output.");
            x.push_str(&format!("You are here: {place}. "));
        }

        if let Some(g) = &self.goal {
            debug!("Appending goal='{g}' to display output.");
            x.push_str(&format!("Goal: {g}."));
        }

        write!(out, "{x}")
    }
}

#[cfg(test)]
mod agent_coordinate_functionality_tests {
    use super::*;

    #[traced_test]
    fn test_here_without_a_goal() {
        let coord = AgentCoordinate::here_without_a_goal("Kitchen");
        assert!(coord.has_location());
        assert!(!coord.has_goal());
        pretty_assert_eq!(*coord.location(), Some("Kitchen".to_string()));
        info!("test_here_without_a_goal passed.");
    }

    #[traced_test]
    fn test_here_with_a_goal() {
        let coord = AgentCoordinate::here_with_a_goal("Office", "Finish report");
        assert!(coord.has_location());
        assert!(coord.has_goal());
        pretty_assert_eq!(*coord.location(), Some("Office".to_string()));
        pretty_assert_eq!(*coord.goal(), Some("Finish report".to_string()));
        info!("test_here_with_a_goal passed.");
    }

    #[traced_test]
    fn test_nowhere_with_no_goal() {
        let coord = AgentCoordinate::nowhere_with_no_goal();
        assert!(!coord.has_location());
        assert!(!coord.has_goal());
        info!("test_nowhere_with_no_goal passed.");
    }

    #[traced_test]
    fn test_nowhere_with_a_goal() {
        let coord = AgentCoordinate::nowhere_with_a_goal("Become world champion");
        assert!(!coord.has_location());
        assert!(coord.has_goal());
        pretty_assert_eq!(*coord.goal(), Some("Become world champion".to_string()));
        info!("test_nowhere_with_a_goal passed.");
    }

    #[traced_test]
    fn test_set_location_and_goal() {
        let mut coord = AgentCoordinate::nowhere_with_no_goal();
        coord.set_location(Some("Library"));
        coord.set_goal(Some("Read more books"));
        pretty_assert_eq!(*coord.location(), Some("Library".to_string()));
        pretty_assert_eq!(*coord.goal(), Some("Read more books".to_string()));
        info!("test_set_location_and_goal passed.");
    }

    #[traced_test]
    fn test_is_fully_specified() {
        let coord1 = AgentCoordinate::here_without_a_goal("Dorm");
        let coord2 = AgentCoordinate::here_with_a_goal("Gym", "Workout");
        assert!(!coord1.is_fully_specified());
        assert!(coord2.is_fully_specified());
        info!("test_is_fully_specified passed.");
    }

    #[traced_test]
    fn test_display() {
        let coord = AgentCoordinate::here_with_a_goal("Park", "Jog around");
        let out = format!("{}", coord);
        assert!(out.contains("Park"));
        assert!(out.contains("Jog around"));
        info!("test_display passed.");
    }

    #[traced_test]
    fn test_builder_usage() {
        let coord = AgentCoordinateBuilder::default()
            .location("School")
            .goal("Attend class")
            .build()
            .expect("Failed to build AgentCoordinate using builder.");
        pretty_assert_eq!(*coord.location(), Some("School".to_string()));
        pretty_assert_eq!(*coord.goal(), Some("Attend class".to_string()));
        info!("test_builder_usage passed.");
    }
}
