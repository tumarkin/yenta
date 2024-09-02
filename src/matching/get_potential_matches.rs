// use rayon::prelude::*;
// use std::collections::BTreeMap;
// use std::marker::Send;

// use crate::core::name::base::HasName;
// use crate::core::{Idf, NameGrouped, NameProcessed, NameUngrouped};
// use crate::matching::types::MatchMode;

// pub trait GetPotentialMatches<M>
// where
//     M: MatchMode<Self>,
//     Self: Sized,
// {
//     type PotentialMatchLookup;

//     fn to_names_weighted(
//         match_mode: &M,
//         ns: Vec<NameProcessed<Self>>,
//         idf: &Idf,
//     ) -> Self::PotentialMatchLookup;

//     fn get_potential_names<'a>(
//         n: &'a Self,
//         pml: &'a Self::PotentialMatchLookup,
//     ) -> Option<&'a Vec<M::MatchableData>>;
// }

// impl<M> GetPotentialMatches<M> for NameUngrouped
// where
//     M: MatchMode<NameUngrouped> + Sync + Sized,
//     M::MatchableData: Send + Sync,
// {
//     type PotentialMatchLookup = Vec<M::MatchableData>;

//     fn to_names_weighted(
//         match_mode: &M,
//         ns: Vec<NameProcessed<Self>>,
//         idf: &Idf,
//     ) -> Self::PotentialMatchLookup {
//         ns.into_par_iter()
//             .map(|name_processed| match_mode.make_matchable_name(name_processed, &idf))
//             .collect()
//     }

//     fn get_potential_names<'a>(
//         _: &'a Self,
//         pml: &'a Self::PotentialMatchLookup,
//     ) -> Option<&'a Vec<M::MatchableData>> {
//         Some(pml)
//     }
// }

// impl<M> GetPotentialMatches<M> for NameGrouped
// where
//     M: MatchMode<NameGrouped> + Sync + Sized,
//     M::MatchableData: Send + Sync,
// {
//     type PotentialMatchLookup = BTreeMap<String, Vec<M::MatchableData>>; // Vec<M::MatchableData>;

//     fn to_names_weighted(
//         match_mode: &M,
//         ns: Vec<NameProcessed<Self>>,
//         idf: &Idf,
//     ) -> Self::PotentialMatchLookup {
//         let mut pml: Self::PotentialMatchLookup = BTreeMap::new();

//         for name_processed in ns {
//             let matchable_name = match_mode.make_matchable_name(name_processed, &idf);
//             let g = matchable_name.get_name().group();
//             let v = pml.entry(g.clone()).or_default();
//             v.push(matchable_name)
//         }

//         println!("{:?}", pml.keys());

//         pml
//         // ns.into_par_iter()
//         //     .map(|name_processed| match_mode.make_matchable_name(name_processed, &idf))
//         //     .collect()
//     }

//     fn get_potential_names<'a>(
//         n: &'a Self,
//         pml: &'a Self::PotentialMatchLookup,
//     ) -> Option<&'a Vec<M::MatchableData>> {
//         pml.get(n.group())
//         // pml
//     }
// }
