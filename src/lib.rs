use std::collections::{HashMap, HashSet, VecDeque};

pub type UnmatchedApplicants<A> = HashSet<A>;

pub type Matches<A, P> = HashMap<A, P>;

/// ## Constraints
/// Max size of program rank order lists: 300 max per <https://www.nrmp.org/help/item/how-many-programs-can-i-rank/>:
///
/// > Rank Order Lists cannot exceed 300
///
/// But we don't need to constrain the number in this implementation.
///
/// Max size of applicant rank order lists: No limit per <https://www.nrmp.org/help/item/how-many-applicants-can-i-rank/>.
/// So, we use a heuristic based on the largest data so far.
/// From <https://www.nrmp.org/about/news/2026/03/nrmp-releases-results-of-the-2026-main-residency-match-for-more-than-38000-future-residents/>,
/// 2026 there were 44,000 spots open. In the pathological space where one hospital has all of that capacity
/// available for everyone, we pick u16 since 2^16 > 44,000.
type ProgramCapacity = u16;

/// Implementation of the Gale-Shapley algorithm to see how fast "The Match" takes depending on the number of people involved.
/// Based on hearsay that matching for fellowship for residents takes only a couple of seconds based on the number of residents,
/// compared to matching for med students or other large populations.
///
/// Given a list of rank order lists between applicants and programs, returns a set of matches
/// (and a set of unmatched applicants) weighted optimally for the applicants.
///
/// Applicants rank programs (Doctor Drew ranks Harvard Hospital first),
/// and programs rank applicants (Carollton Care ranks Doctor Drew second).
///
/// Optimization: when swapping a candidate, tell them where we kicked them from so we don't have to traverse up to that point again.
/// Optimization: find connected components first then run this algorithm on the independent components?
///
/// TODO: <https://www.nrmp.org/residency-applicants/get-ready-for-the-match/couples-in-the-match/>
pub fn match_algorithm<A: Clone + Eq + std::hash::Hash, P: Clone + Eq + std::hash::Hash>(
    program_capacities: HashMap<P, ProgramCapacity>,
    program_rank_order_lists: HashMap<A, Vec<P>>,
    applicant_rank_order_lists: HashMap<P, Vec<A>>,
) -> (Matches<A, P>, UnmatchedApplicants<A>) {
    let mut unmatched_applicants = HashSet::new();

    let mut rankings = Rankings::new(&program_capacities, &applicant_rank_order_lists);

    let mut candidates = program_rank_order_lists
        .keys()
        .cloned()
        .collect::<VecDeque<_>>();

    'candidate: while let Some(candidate) = candidates.pop_front() {
        'program: for program in &program_rank_order_lists[&candidate] {
            match rankings.attempt_match(&candidate, program) {
                MatchResult::MatchedWithCapacity => {
                    continue 'candidate;
                }
                MatchResult::WillSwapFor(other_candidate) => {
                    candidates.push_back(other_candidate);
                    continue 'candidate;
                }
                MatchResult::NotInterested => {
                    continue 'program;
                }
            }
        }

        unmatched_applicants.insert(candidate);
    }

    (rankings.matches(), unmatched_applicants)
}

enum MatchResult<A> {
    MatchedWithCapacity,
    WillSwapFor(A),
    // Program either did not list applicant or is at capacity and prefers all existing matches.
    NotInterested,
}

struct Rankings<'input, P, A> {
    program_capacities: &'input HashMap<P, ProgramCapacity>,
    applicant_rank_order_lists: &'input HashMap<P, Vec<A>>,
    ranked_matches: HashMap<P, Vec<(A, ProgramCapacity)>>,
}

impl<'i, P, A> Rankings<'i, P, A>
where
    P: Eq + std::hash::Hash + Clone,
    A: Eq + std::hash::Hash + Clone,
{
    fn new(
        program_capacities: &'i HashMap<P, ProgramCapacity>,
        applicant_rank_order_lists: &'i HashMap<P, Vec<A>>,
    ) -> Self {
        Rankings {
            program_capacities,
            applicant_rank_order_lists,
            ranked_matches: HashMap::new(),
        }
    }

    fn attempt_match(&mut self, applicant: &A, program: &P) -> MatchResult<A> {
        let program_ranked_applicant = {
            self.applicant_rank_order_lists
                .get(program)
                .and_then(|rol| {
                    rol.iter()
                        .position(|ranked_applicant| ranked_applicant == applicant)
                })
        };
        let Some(program_ranked_applicant) = program_ranked_applicant else {
            return MatchResult::NotInterested;
        };
        let program_ranked_applicant = program_ranked_applicant
            .try_into()
            .expect("higher number of applicants than expected");
        let program_matches = { self.ranked_matches.entry(program.clone()).or_default() };
        let Some(max_program_capacity) = self.program_capacities.get(program) else {
            return MatchResult::NotInterested;
        };
        if *max_program_capacity == 0 {
            return MatchResult::NotInterested;
        }
        if program_matches.len() < (*max_program_capacity as usize) {
            program_matches.push((applicant.clone(), program_ranked_applicant));
            return MatchResult::MatchedWithCapacity;
        }

        // See if the program would rather have someone else.
        // Find the min, compare the rankings, then either replace if new applicant is better
        // or say not interested.
        let (least_favorite_index, (_, least_favorite_ranking)) = {
            program_matches
                .iter()
                .enumerate()
                .max_by_key(|(_, (_, ranking))| ranking)
                .expect("capacity is non-zero and length is greater than capacity")
        };

        if *least_favorite_ranking < program_ranked_applicant {
            MatchResult::NotInterested
        } else {
            let least_favorite_applicant = program_matches.swap_remove(least_favorite_index);
            program_matches.push((applicant.clone(), program_ranked_applicant));
            MatchResult::WillSwapFor(least_favorite_applicant.0)
        }
    }

    fn matches(self) -> Matches<A, P> {
        self.ranked_matches
            .into_iter()
            .fold(HashMap::new(), |mut acc, program_rankings| {
                let (program, applicants) = program_rankings;
                let accepted_applicants = applicants
                    .into_iter()
                    .map(|(applicant, _)| (applicant, program.clone()));
                acc.extend(accepted_applicants);
                acc
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wikipedia_example() {
        let doctor_a = "A";
        let doctor_b = "B";
        let doctor_c = "C";

        let hospital_x = "X";
        let hospital_y = "Y";
        let hospital_z = "Z";

        // A: YXZ   B: ZYX   C: XZY
        let doctor_rankings = [
            (doctor_a, vec![hospital_y, hospital_x, hospital_z]),
            (doctor_b, vec![hospital_z, hospital_y, hospital_x]),
            (doctor_c, vec![hospital_x, hospital_z, hospital_y]),
        ]
        .into_iter()
        .collect();

        // X: BAC   Y: CBA   Z: ACB
        let hospital_rankings = [
            (hospital_x, vec![doctor_b, doctor_a, doctor_c]),
            (hospital_y, vec![doctor_c, doctor_b, doctor_a]),
            (hospital_z, vec![doctor_a, doctor_c, doctor_b]),
        ]
        .into_iter()
        .collect();

        let hospital_capacities = [(hospital_x, 1), (hospital_y, 1), (hospital_z, 1)]
            .into_iter()
            .collect();

        // possible solutions
        let solution = match_algorithm(hospital_capacities, doctor_rankings, hospital_rankings);
        // doctors get their first choice and hospitals their third – (AY, BZ, CX);
        assert!(
            solution
                == (
                    HashMap::from_iter([
                        (doctor_a, hospital_y),
                        (doctor_b, hospital_z),
                        (doctor_c, hospital_x)
                    ]),
                    HashSet::new()
                ),
            // || solution
            //     == (
            //         HashMap::from_iter([
            //             // everyone gets their second choice – (AX, BY, CZ);
            //             (doctor_a.clone(), hospital_x.clone()),
            //             (doctor_b.clone(), hospital_y.clone()),
            //             (doctor_c.clone(), hospital_z.clone())
            //         ],),
            //         HashSet::new()
            //     )
            // || solution
            //     == (
            //         HashMap::from_iter([
            //             // hospitals get their first choice and doctors their third – (AZ, BX, CY)
            //             (doctor_a.clone(), hospital_z.clone()),
            //             (doctor_b.clone(), hospital_x.clone()),
            //             (doctor_c.clone(), hospital_y.clone())
            //         ]),
            //         HashSet::new(),
            //     ),
            "unexpected solution: {:?}",
            solution
        );
        // If we have two rings, make progress on both separately? So there would be nine actual states: for each state of the first ring, the second ring could be in one of three states.
    }

    /// From <https://www.youtube.com/watch?v=yieyXPdsdsk>
    #[test]
    fn youtube_example() {
        let andre = "Andre";
        let paul = "Paul";
        let jordan = "Jordan";
        let teresa = "Teresa";
        let omar = "Omar";
        let allison = "Allison";

        let mercy = "Mercy";
        let city = "City";
        let general = "General";

        // A: YXZ   B: ZYX   C: XZY
        let doctor_rankings = [
            (andre, vec![city]),
            (paul, vec![city, mercy, general]),
            (jordan, vec![city, mercy, general]),
            (teresa, vec![mercy, city, general]),
            (omar, vec![mercy, general, city]),
            (allison, vec![city, general, mercy]),
        ]
        .into_iter()
        .collect();

        // X: BAC   Y: CBA   Z: ACB
        let hospital_rankings = [
            (mercy, vec![andre, jordan]),
            (city, vec![allison, omar, andre, teresa, paul, jordan]),
            (general, vec![omar, allison, andre, teresa, paul, jordan]),
        ]
        .into_iter()
        .collect();

        let hospital_capacities = [(mercy, 2), (city, 2), (general, 2)].into_iter().collect();

        // possible solutions
        let solution = match_algorithm(hospital_capacities, doctor_rankings, hospital_rankings);
        // doctors get their first choice and hospitals their third – (AY, BZ, CX);
        assert!(
            solution
                == (
                    HashMap::from_iter([
                        (andre, city),
                        (jordan, mercy),
                        (teresa, general),
                        (omar, general),
                        (allison, city),
                    ]),
                    HashSet::from_iter([paul])
                ),
            "unexpected solution: {:?}",
            solution
        );
    }

    // properties:
    // - no duplicate doctors in match output
    // - for all matchings, if a doctor prefers a hospital over their current match, that hospital prefers the doctor lower than their current match
    //
    // How to generate good, arbitrary rank lists though?
}
