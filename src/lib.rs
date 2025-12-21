use std::collections::{HashMap, HashSet, VecDeque};

pub type UnmatchedApplicants<A> = HashSet<A>;

pub type Matches<A, P> = HashMap<A, P>;

/// Applicant rank order lists. No limit per <https://www.nrmp.org/help/item/how-many-applicants-can-i-rank/>.
/// So, we use a heuristic based on the largest data so far.
/// From <https://www.nrmp.org/about/news/2026/03/nrmp-releases-results-of-the-2026-main-residency-match-for-more-than-38000-future-residents/>,
/// 2026 there were 44,000 spots open. In the pathological space where one hospital has all of that capacity,
/// available for everyone, we pick u16 since 2^16 > 44,000.
type ProgramCapacity = u16;

/// Program rank order lists. 300 max per <https://www.nrmp.org/help/item/how-many-programs-can-i-rank/>:
///
/// > Rank Order Lists cannot exceed 300
///
/// 2^9 is 512, so that would work, but Rust doesn't have arbitrary sized integers
/// (and I don't want to switch to the only language I know that does, Zig),
/// so we are going with u16. In practice, u8 would probably work...
type ApplicantRanking = u16;

/// Implementation of the Gale-Shapley algorithm to see how fast "The Match" takes depending on the number of people involved.
/// Based on hearsay that matching for fellowship for residents takes only a couple of seconds based on the number of residents,
/// compared to matching for med students or other large populations.
///
/// Given a list of rank order lists between applicants and programs, returns a set of matches
/// (and a set of unmatched applicants) weighted optimally for the applicants.
///
/// Applicants rank programs (Doctor Drew ranks Harvard Hospital first),
/// and programs rank applicants (Carollton Care ranks Doctor Drew second).
pub fn match_algorithm<A: Clone + Eq + std::hash::Hash, P>(
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

    while let Some(candidate) = candidates.pop_front() {
        for program in &program_rank_order_lists[&candidate] {
            match rankings.attempt_match(&candidate, program) {
                MatchResult::MatchedWithCapacity => {
                    break;
                }
                MatchResult::WillSwapFor(other_candidate) => {
                    candidates.push_back(other_candidate);
                    break;
                }
                MatchResult::NotInterested => {
                    continue;
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
}

impl<'i, P, A> Rankings<'i, P, A> {
    fn new(
        program_capacities: &'i HashMap<P, ProgramCapacity>,
        applicant_rank_order_lists: &'i HashMap<P, Vec<A>>,
    ) -> Self {
        Rankings {
            program_capacities,
            applicant_rank_order_lists,
        }
    }

    fn attempt_match(&mut self, applicant: &A, program: &P) -> MatchResult<A> {
        MatchResult::MatchedWithCapacity
    }

    fn matches(self) -> Matches<A, P> {
        HashMap::new()
    }
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
                == (
                    HashSet::from_iter([
                        Match(doctor_a.clone(), hospital_y.clone()),
                        Match(doctor_b.clone(), hospital_z.clone()),
                        Match(doctor_c.clone(), hospital_x.clone())
                    ]),
                    HashSet::new()
                )
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
        // If we have two rings, make progress on both separately? So there would be nine actual states: for each state of the first ring, the second ring could be in one of three states.
    }

    // properties:
    // - no duplicate doctors in match output
    // - for all matchings, if a doctor prefers a hospital over their current match, that hospital prefers the doctor lower than their current match
    //
    // How to generate good, arbitrary rank lists though?
}
