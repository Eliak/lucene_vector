use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use jni::objects::JObject;
use jni::sys::jfloatArray;
use jni::JNIEnv;
use packed_simd::{f32x16, f32x4, f32x8};

// type Cache = Arc<RwLock<HashMap<i32, Arc<Vec<f32>>, BuildHasherDefault<FNV1aHasher32>>>>;
// type Cache = Arc<RwLock<HashMap<i32, Arc<Vec<f32>>, BuildHasherDefault<FxHasher32>>>>;
type Cache = Arc<RwLock<HashMap<i32, Arc<Vec<f32>>>>>;

fn new_cache() -> Cache {
    // Arc::new(RwLock::new(HashMap::with_capacity_and_hasher(1000, BuildHasherDefault::<FNV1aHasher32>::default())))
    // Arc::new(RwLock::new(HashMap::with_capacity_and_hasher(1000, BuildHasherDefault::<FxHasher32>::default())))
    Arc::new(RwLock::new(HashMap::with_capacity(1000)))
}

struct ScorerFactory {
    cache: Cache,
}

impl ScorerFactory {
    pub fn new() -> ScorerFactory {
        ScorerFactory { cache: new_cache() }
    }
    pub fn scorer(&self, query_vector: Vec<f32>) -> Scorer {
        Scorer {
            query_vector: Box::new(query_vector),
            cache: self.cache.clone(),
        }
    }
}

struct Scorer {
    query_vector: Box<Vec<f32>>,
    cache: Cache,
}

impl Scorer {
    pub fn score(&self, env: &JNIEnv, doc_id: i32, callback: JObject) -> f32 {
        let vector: Arc<Vec<f32>> = self.vector(env, doc_id, callback);
        cosine_similarity(self.query_vector.as_ref(), vector.as_ref())
    }

    fn vector(&self, env: &JNIEnv, doc_id: i32, callback: JObject) -> Arc<Vec<f32>> {
        // return VEC_DUMMY.clone();
        let cache = self.cache.clone();
        {
            let guard = cache.read().unwrap();
            if let Some(v) = guard.get(&doc_id) {
                return v.clone();
            }
        }
        let result = env.call_method(callback, "binaryValue", "()[F", &[]);
        if result.is_err() {
            panic!("receive binaryValue error: {}", result.err().unwrap());
        }
        let b_array = result.unwrap().l().unwrap().into_inner() as jfloatArray;
        let vec: Arc<Vec<f32>> = Arc::new(convert_to_vec(env, b_array));
        {
            let mut guard = cache.write().unwrap();
            guard.insert(doc_id.clone(), vec.clone());
        }
        return vec;
    }
}

pub fn convert_to_vec(env: &JNIEnv, array: jfloatArray) -> Vec<f32> {
    let len = env.get_array_length(array).unwrap();
    let mut vec = vec![0f32; len as usize];
    env.get_float_array_region(array, 0, vec.as_mut()).unwrap();
    return vec;
}

pub fn cosine_similarity(one: &[f32], another: &[f32]) -> f32 {
    assert_eq!(one.len(), another.len());
    let size = one.len() - 1;
    let dot_product: f32 = dot_prod(&one[..size], &another[..size]);
    return dot_product / (one[size] * another[size]);
}

pub fn dot_prod(a: &[f32], b: &[f32]) -> f32 {
    dot_prod16(a, b)
}

pub fn dot_prod1(a: &[f32], b: &[f32]) -> f32 {
    let mut dot_product: f32 = 0f32;
    for i in 0..a.len() {
        dot_product += a[i] * b[i];
    }
    return dot_product;
}

pub fn dot_prod4(a: &[f32], b: &[f32]) -> f32 {
    assert_eq!(a.len(), b.len());
    assert_eq!(a.len() % 4, 0);

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
