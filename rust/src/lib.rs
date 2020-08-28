#![feature(test)]

extern crate test;
extern crate packed_simd;

use std::collections::HashMap;
use std::iter::Map;
use std::mem::transmute;
use std::rc::Rc;
use std::sync::{Arc, RwLock};
use std::time::Instant;

use jni::JNIEnv;
use jni::objects::{JClass, JObject, JValue};
use jni::sys::{jbyteArray, jfloatArray, jint, jlong};
use packed_simd::{f32x4, f32x8, f32x16};
use rand::Rng;

struct ScorerFactory {
    cache:Arc<RwLock<HashMap<i32, Rc<Vec<f32>>>>>
}
impl ScorerFactory {
    pub fn new() -> ScorerFactory {
        ScorerFactory {
            cache: Arc::new(RwLock::new(HashMap::new()))
        }
    }
    pub fn scorer(mut self:&ScorerFactory, query_vector:Vec<f32>) -> Scorer {
        Scorer {
            query_vector: Box::new(query_vector),
            cache: self.cache.clone()
        }
    }
}
struct Scorer {
    query_vector:Box<Vec<f32>>,
    cache:Arc<RwLock<HashMap<i32, Rc<Vec<f32>>>>>
}

impl Scorer {
    pub fn score(&self, env: &JNIEnv, doc_id: i32, callback: JObject) -> f32 {
        let vector: Rc<Vec<f32>> = self.vector(env, doc_id, callback);
        cosine_similarity(self.query_vector.as_ref(), vector.as_ref())
    }

    fn vector(&self, env: &JNIEnv, doc_id: i32, callback: JObject) -> Rc<Vec<f32>> {
        let cache = self.cache.clone();
        {
            let guard = cache.read().unwrap();
            let option = guard.get(&doc_id);
            if option.is_some() {
                return option.unwrap().clone();
            }
        }
        let result = env.call_method(callback, "binaryValue", "()[F", &[]);
        if(result.is_err()) {
            panic!("receive binaryValue error: {}", result.err().unwrap());
        }
        let b_array = result.unwrap().l().unwrap().into_inner() as jfloatArray;
        let vec: Vec<f32> = convert_to_vec(env, b_array);
        cache.write().unwrap().insert(doc_id.clone(), Rc::new(vec));
        return cache.read().unwrap().get(&doc_id).unwrap().clone();
    }
}

/*
 * Class:     com_github_eliak_VScoreNative
 * Method:    cosineSimilarity
 * Signature: ([F[F)F
 */
#[no_mangle]
pub extern "system" fn Java_com_github_eliak_VScoreNative_cosineSimilarity(
    env: JNIEnv, _class: JClass,
    one: jfloatArray, another: jfloatArray
) -> f32 {
    let vec1 = convert_to_vec(&env, one);
    let vec2 = convert_to_vec(&env, another);
    let similarity = cosine_similarity(&vec1, &vec2);
    drop(vec1);
    drop(vec2);
    return similarity;
}

/*
 * Class:     com_github_eliak_VScoreNative
 * Method:    createScorerFactory
 * Signature: ()J
 */
#[no_mangle]
pub unsafe extern "system" fn Java_com_github_eliak_VScoreNative_createScorerFactory(env: JNIEnv, _class: JClass) -> i64 {
    let factory = ScorerFactory::new();
    let result = Box::into_raw(Box::new(factory)) as jlong;
    // println!("createScorerFactory: {}", result);
    result
}

/*
 * Class:     com_github_eliak_VScoreNative
 * Method:    destroyScorerFactory
 * Signature: (J)J
 */
#[no_mangle]
pub unsafe extern "system" fn Java_com_github_eliak_VScoreNative_destroyScorerFactory(
    env: JNIEnv, _class: JClass,
    factory_ptr: jlong
) {
    // println!("destroyScorerFactory: {}", factory_ptr);
    let _boxed_factory = Box::from_raw(factory_ptr as *mut ScorerFactory);
    drop(_boxed_factory);
}

/*
 * Class:     com_github_eliak_VScoreNative
 * Method:    createScorer
 * Signature: (J[F)J
 */
#[no_mangle]
pub unsafe extern "system" fn Java_com_github_eliak_VScoreNative_createScorer(
    env: JNIEnv, _class: JClass,
    factory_ptr: jlong, query_vector: jfloatArray
) -> jlong {
    let factory = &*(factory_ptr as *const ScorerFactory);
    let scorer = factory.scorer(convert_to_vec(&env, query_vector));
    let result = Box::into_raw(Box::new(scorer)) as jlong;
    // println!("createScorer: {} from factory {}, cache.len={}", result, factory_ptr, factory.cache.clone().read().unwrap().len());
    result
}

/*
 * Class:     com_github_eliak_VScoreNative
 * Method:    destroyScorer
 * Signature: (J)V
 */
#[no_mangle]
pub unsafe extern "system" fn Java_com_github_eliak_VScoreNative_destroyScorer(
    env: JNIEnv, _class: JClass,
    scorer_ptr: jlong
) {
    // println!("destroyScorer: {}", scorer_ptr);
    let _boxed_scorer = Box::from_raw(scorer_ptr as *mut Scorer);
    drop(_boxed_scorer);
}

/*
 * Class:     com_github_eliak_VScoreNative
 * Method:    score
 * Signature: (JILcom/github/eliak/VScoreNative/ScorerCallback;)F
 */
#[no_mangle]
pub unsafe extern "system" fn Java_com_github_eliak_VScoreNative_score(
    env: JNIEnv, _class: JClass,
    scorer_ptr: jlong, doc_id: jint, callback: JObject
) -> f32 {
    let scorer = &*(scorer_ptr as *const Scorer);
    scorer.score(&env, doc_id, callback)
}



fn convert_to_vec(env: &JNIEnv, array: jfloatArray) -> Vec<f32> {
    let len = env.get_array_length(array).unwrap();
    let mut vec = vec![0f32; len as usize];
    env.get_float_array_region(array, 0, vec.as_mut()).unwrap();
    return vec;
}

fn cosine_similarity(one:&[f32], another:&[f32]) -> f32 {
    assert_eq!(one.len(), another.len());
    let size = one.len() - 1;
    let dot_product:f32 = dot_prod8(&one[..size], &another[..size]);
    return dot_product / (one[size] * another[size]);
}

pub fn dot_prod1(a: &[f32], b: &[f32]) -> f32 {
    let mut dot_product:f32 = 0f32;
    for i in 0..a.len() {
        dot_product += a[i] * b[i];
    }
    return dot_product;
}

pub fn dot_prod4(a: &[f32], b: &[f32]) -> f32 {
    assert_eq!(a.len(), b.len());
    assert!(a.len() % 4 == 0);

    a.chunks_exact(4)
        .map(f32x4::from_slice_unaligned)
        .zip(b.chunks_exact(4).map(f32x4::from_slice_unaligned))
        .map(|(a, b)| a * b)
        .sum::<f32x4>()
        .sum()
}
pub fn dot_prod8(a: &[f32], b: &[f32]) -> f32 {
    assert_eq!(a.len(), b.len());
    assert!(a.len() % 8 == 0);

    a.chunks_exact(8)
        .map(f32x8::from_slice_unaligned)
        .zip(b.chunks_exact(8).map(f32x8::from_slice_unaligned))
        .map(|(a, b)| a * b)
        .sum::<f32x8>()
        .sum()
}

pub fn dot_prod16(a: &[f32], b: &[f32]) -> f32 {
    assert_eq!(a.len(), b.len());
    assert!(a.len() % 16 == 0);

    a.chunks_exact(16)
        .map(f32x16::from_slice_unaligned)
        .zip(b.chunks_exact(16).map(f32x16::from_slice_unaligned))
        .map(|(a, b)| a * b)
        .sum::<f32x16>()
        .sum()
}

fn generate_array(size:usize) -> Vec<f32> {
    let mut vec = Vec::new();
    let mut rng = rand::thread_rng();
    let mut dot_product:f64 = 0 as f64;
    for _ in 0..size {
        let val = rng.gen::<f32>();
        dot_product += (val as f64).powi(2);
        vec.push(val);
    }
    vec.push(dot_product.sqrt() as f32);
    return vec;
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use crate::{cosine_similarity, generate_array};
    use test::Bencher;

    #[test]
    fn test_cosine_similarity() {
        let vec = generate_array(64);
        assert_eq!((cosine_similarity(&vec, &vec) * 10000f32).round(), 10000f32);
    }

    /**
     * Настоящий бенчмарк выдаёт малопонятный результат типа
     *   test tests::bench_cosine_similarity2 ... bench:  65,161,520 ns/iter (+/- 5,057,415)
     *   test result: ok. 0 passed; 0 failed; 0 ignored; 1 measured; 2 filtered out
     * по этому написал такое:
     */
    #[test]
    fn bench_cosine_similarity() {
        let vec = generate_array(512);
        let start = Instant::now();
        let mut similarity: f32 = 0f32;
        let size = 1000000;
        for i in 0..size {
            similarity += cosine_similarity(&vec, &vec);
        }
        let duration = Instant::now().checked_duration_since(start).unwrap();
        println!("{}", similarity);
        let d = duration.as_secs_f64();
        println!("duration {}, {} op/s", d, size as f64 / d);
    }

    #[bench]
    fn bench_cosine_similarity2(b:&mut Bencher) {
        let vec = generate_array(512);
        b.iter(|| {
            let size = test::black_box(1000000);
            let mut similarity: f32 = 0f32;
            for i in 0..size {
                similarity += cosine_similarity(&vec, &vec);
            }
        });
    }
}
