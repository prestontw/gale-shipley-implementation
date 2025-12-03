use std::collections::HashSet;

/// Implementation of the Gale-Shapley algorithm to see how fast "The Match" takes depending on the number of people involved.
/// Based on hearsay that matching for fellowship for residents takes only a couple of seconds based on the number of residents,
/// compared to matching for med students or other large populations.

#[derive(Clone, Hash, Eq, PartialEq, Debug)]
pub struct Doctor(String);
impl Doctor {
    pub fn new(name: &str) -> Self {
        Doctor(name.to_string())
    }
}

#[derive(Clone, Hash, Eq, PartialEq, Debug)]
pub struct Hospital(String);
impl Hospital {
    pub fn new(name: &str) -> Self {
        Hospital(name.to_string())
    }
}

// Represents a doctor's rank list, where the earlier elements in the list are the doctor's higher priorities.
#[derive(Clone)]
pub struct DoctorsRankList(Vec<Hospital>);

impl DoctorsRankList {
    pub fn new(hospital_rankings: Vec<Hospital>) -> Self {
        DoctorsRankList(hospital_rankings)
    }
}

#[derive(Clone)]
pub struct HospitalsRankList(Vec<Doctor>);

impl HospitalsRankList {
    pub fn new(doctor_rankings: Vec<Doctor>) -> Self {
        HospitalsRankList(doctor_rankings)
    }
}

#[derive(Eq, PartialEq, Hash, Debug)]
pub struct Match(Doctor, Hospital);

pub fn match_algorithm(
    doctor_ranking_lists: Vec<(Doctor, DoctorsRankList)>,
    hospital_ranking_lists: Vec<(Hospital, HospitalsRankList)>,
) -> HashSet<Match> {
    HashSet::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wikipedia_example() {
        let doctor_a = Doctor::new("A");
        let doctor_b = Doctor::new("B");
        let doctor_c = Doctor::new("C");

        let hospital_x = Hospital::new("X");
        let hospital_y = Hospital::new("Y");
        let hospital_z = Hospital::new("Z");

        // A: YXZ   B: ZYX   C: XZY
        let doctor_rankings = [
            (
                doctor_a.clone(),
                DoctorsRankList::new(vec![
                    hospital_y.clone(),
                    hospital_x.clone(),
                    hospital_z.clone(),
                ]),
            ),
            (
                doctor_b.clone(),
                DoctorsRankList::new(vec![
                    hospital_z.clone(),
                    hospital_y.clone(),
                    hospital_x.clone(),
                ]),
            ),
            (
                doctor_c.clone(),
                DoctorsRankList::new(vec![
                    hospital_x.clone(),
                    hospital_z.clone(),
                    hospital_y.clone(),
                ]),
            ),
        ]
        .to_vec();
        // X: BAC   Y: CBA   Z: ACB
        let hospital_rankings = [
            (
                hospital_x.clone(),
                HospitalsRankList::new(vec![doctor_b.clone(), doctor_a.clone(), doctor_c.clone()]),
            ),
            (
                hospital_y.clone(),
                HospitalsRankList::new(vec![doctor_c.clone(), doctor_b.clone(), doctor_a.clone()]),
            ),
            (
                hospital_z.clone(),
                HospitalsRankList::new(vec![doctor_a.clone(), doctor_c.clone(), doctor_b.clone()]),
            ),
        ]
        .to_vec();

        // possible solutions
        let solution = match_algorithm(doctor_rankings, hospital_rankings);
        // doctors get their first choice and hospitals their third – (AY, BZ, CX);
        assert!(
            solution
                == HashSet::from_iter([
                    Match(doctor_a.clone(), hospital_y.clone()),
                    Match(doctor_b.clone(), hospital_z.clone()),
                    Match(doctor_c.clone(), hospital_x.clone())
                ])
                || solution
                    == HashSet::from_iter([
                        // everyone gets their second choice – (AX, BY, CZ);
                        Match(doctor_a.clone(), hospital_x.clone()),
                        Match(doctor_b.clone(), hospital_y.clone()),
                        Match(doctor_c.clone(), hospital_z.clone())
                    ])
                || solution
                    == HashSet::from_iter([
                        // hospitals get their first choice and doctors their third – (AZ, BX, CY)
                        Match(doctor_a.clone(), hospital_z.clone()),
                        Match(doctor_b.clone(), hospital_x.clone()),
                        Match(doctor_c.clone(), hospital_y.clone())
                    ]),
            "unexpected solution: {:?}",
            solution
        );
    }
}
