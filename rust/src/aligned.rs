use jni::objects::JObject;
use jni::sys::{jfloatArray, jsize};
use jni::JNIEnv;
use packed_simd::{f32x16, f32x4, f32x8};
use rand::Rng;
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use std::sync::{Arc, RwLock};

const SIZE_VECTOR: usize = 512;
const SIZE_VECTOR_AS_JSIZE: jsize = SIZE_VECTOR as jsize;

pub type DocId = i64;

// type Cache = Arc<RwLock<HashMap<i32, Arc<Vec<f32>>, BuildHasherDefault<FNV1aHasher32>>>>;
// type Cache = Arc<RwLock<HashMap<i32, Arc<Vec<f32>>, BuildHasherDefault<FxHasher32>>>>;
pub type Cache = Arc<RwLock<HashMap<DocId, Arc<Item>>>>;

fn new_cache() -> Cache {
    // Arc::new(RwLock::new(HashMap::with_capacity_and_hasher(1000, BuildHasherDefault::<FNV1aHasher32>::default())))
    // Arc::new(RwLock::new(HashMap::with_capacity_and_hasher(1000, BuildHasherDefault::<FxHasher32>::default())))
    Arc::new(RwLock::new(HashMap::with_capacity(1000)))
}

#[repr(C, align(64))]
struct Vector([f32; SIZE_VECTOR]);

impl Deref for Vector {
    type Target = [f32; SIZE_VECTOR];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Vector {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Vector {
    pub fn new() -> Vector {
        Vector([0f32; SIZE_VECTOR])
    }

    pub fn doc_product(&self, other: &Vector) -> f32 {
        return self.dot_product_16(other);
    }

    pub fn dot_product_with_unaligned(&self, slice: &[f32]) -> f32 {
        assert_eq!(slice.len(), SIZE_VECTOR);
        self.chunks_exact(16)
            .map(f32x16::from_slice_aligned)
            .zip(slice.chunks_exact(16).map(f32x16::from_slice_unaligned))
            .map(|(a, b)| a * b)
            .sum::<f32x16>()
            .sum()
    }

    #[inline]
    fn doc_product_base(&self, another: &Vector) -> f32 {
        let mut dot_product: f32 = 0f32;
        for i in 0..SIZE_VECTOR {
            dot_product += self[i] * another[i];
        }
        return dot_product;
    }

    #[inline]
    fn dot_product_4(&self, another: &Vector) -> f32 {
        self.chunks_exact(4)
            .map(f32x4::from_slice_aligned)
            .zip(another.chunks_exact(4).map(f32x4::from_slice_aligned))
            .map(|(a, b)| a * b)
            .sum::<f32x4>()
            .sum()
    }

    #[inline]
    fn dot_product_8(&self, another: &Vector) -> f32 {
        self.chunks_exact(8)
            .map(f32x8::from_slice_aligned)
            .zip(another.chunks_exact(8).map(f32x8::from_slice_aligned))
            .map(|(a, b)| a * b)
            .sum::<f32x8>()
            .sum()
    }

    #[inline]
    fn dot_product_16(&self, another: &Vector) -> f32 {
        self.chunks_exact(16)
            .map(f32x16::from_slice_aligned)
            .zip(another.chunks_exact(16).map(f32x16::from_slice_aligned))
            .map(|(a, b)| a * b)
            .sum::<f32x16>()
            .sum()
    }
}

pub struct Item {
    vector: Vector,
    magnitude: f32,
}

impl Item {
    pub fn new() -> Item {
        Item {
            vector: Vector::new(),
            magnitude: 0f32,
        }
    }

    pub fn random() -> Item {
        let mut item = Item::new();
        item.fill_random();
        return item;
    }

    pub fn from_jni_float_array(env: &JNIEnv, array: jfloatArray) -> Item {
        let len = env.get_array_length(array).unwrap();
        if len < SIZE_VECTOR_AS_JSIZE {
            panic!(
                "array length {:?} is lower then required {:?}",
                len, SIZE_VECTOR_AS_JSIZE
            );
        }
        let mut item = Item::new();
        env.get_float_array_region(array, 0, item.vector.as_mut())
            .unwrap();
        if len == SIZE_VECTOR_AS_JSIZE {
            let mut dot_product: f64 = 0f64;
            for i in 0..SIZE_VECTOR {
                dot_product += (item.vector[i] as f64).powi(2);
            }
            item.magnitude = dot_product.sqrt() as f32;
        } else if len + 1 == SIZE_VECTOR_AS_JSIZE {
            let mut magnitude = [0f32];
            env.get_float_array_region(array, SIZE_VECTOR_AS_JSIZE, magnitude.as_mut())
                .unwrap();
            item.magnitude = magnitude[0];
        } else {
            panic!(
                "array length {:?} is greater then required {:?}",
                len, SIZE_VECTOR_AS_JSIZE
            );
        }
        return item;
    }

    pub fn fill_random(&mut self) {
        let mut rng = rand::thread_rng();
        let mut dot_product: f64 = 0f64;
        for i in 0..SIZE_VECTOR {
            let val = rng.gen::<f32>();
            dot_product += (val as f64).powi(2);
            self.vector[i] = val;
        }
        self.magnitude = dot_product.sqrt() as f32;
    }

    pub fn dot_product(&self, another: &Item) -> f32 {
        self.vector.doc_product(&another.vector)
    }

    pub fn dot_product_with_unaligned(&self, another: &[f32]) -> f32 {
        self.vector.dot_product_with_unaligned(another)
    }

    pub fn cosine_similarity(&self, another: &Item) -> f32 {
        return self.dot_product(another) / (self.magnitude * another.magnitude);
    }
}

pub struct ScorerFactory {
    pub(crate) cache: Cache,
}

impl ScorerFactory {
    pub fn new() -> ScorerFactory {
        ScorerFactory { cache: new_cache() }
    }
    pub fn scorer(&self, query_vector: Item) -> Scorer {
        Scorer {
            query_vector: Box::new(query_vector),
            cache: self.cache.clone(),
        }
    }
}

pub struct Scorer {
    query_vector: Box<Item>,
    cache: Cache,
}

impl Scorer {
    pub fn score(&self, env: &JNIEnv, doc_id: DocId, callback: JObject) -> f32 {
        self.cosine_similarity(env, doc_id, callback)
    }

    pub fn dot_product(&self, env: &JNIEnv, doc_id: DocId, callback: JObject) -> f32 {
        let item: Arc<Item> = self.item(env, doc_id, callback);
        self.query_vector.dot_product(item.as_ref())
    }

    pub fn cosine_similarity(&self, env: &JNIEnv, doc_id: DocId, callback: JObject) -> f32 {
        let item: Arc<Item> = self.item(env, doc_id, callback);
        self.query_vector.cosine_similarity(item.as_ref())
    }

    fn item(&self, env: &JNIEnv, doc_id: DocId, callback: JObject) -> Arc<Item> {
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
        let vec: Arc<Item> = Arc::new(Item::from_jni_float_array(env, b_array));
        {
            let mut guard = cache.write().unwrap();
            guard.insert(doc_id.clone(), vec.clone());
        }
        return vec;
    }
}
