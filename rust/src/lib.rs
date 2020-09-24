#![feature(test)]
#![feature(type_name_of_val)]

#[macro_use]
extern crate lazy_static;
extern crate packed_simd;
extern crate test;

use std::collections::HashMap;
use std::hash::BuildHasherDefault;
use std::ops::{Deref, DerefMut};
use std::sync::{Arc, RwLock};

use hashers::fnv::FNV1aHasher32;
use hashers::fx_hash::FxHasher32;
use jni::objects::{JClass, JObject, ReleaseMode};
use jni::sys::{jbyte, jbyteArray, jfloat, jfloatArray, jint, jlong, jsize};
use jni::JNIEnv;
use packed_simd::{f32x16, f32x4, f32x8};
use rand::Rng;

mod aligned;
mod unaligned;

#[cfg(test)]
mod tests;

/*
 * Class:     com_github_eliak_VScoreNative
 * Method:    cosineSimilarity
 * Signature: ([F[F)F
 */
#[no_mangle]
pub extern "system" fn Java_com_github_eliak_VScoreNative_cosineSimilarity(
    _env: JNIEnv,
    _class: JClass,
    one: jfloatArray,
    another: jfloatArray,
) -> f32 {
    let item1 = aligned::Item::from_jni_float_array(&_env, one);
    let item2 = aligned::Item::from_jni_float_array(&_env, another);
    let similarity = item1.cosine_similarity(&item2);
    drop(item1);
    drop(item2);
    return similarity;
}

/*
 * Class:     com_github_eliak_VScoreNative
 * Method:    cosineSimilarity2
 * Signature: ([F[F)F
 */
#[no_mangle]
pub extern "system" fn Java_com_github_eliak_VScoreNative_cosineSimilarity2(
    _env: JNIEnv,
    _class: JClass,
    one: jfloatArray,
    two: jfloatArray,
) -> f32 {
    let one_len = _env.get_array_length(one).unwrap();
    let two_len = _env.get_array_length(two).unwrap();
    return Java_com_github_eliak_VScoreNative_cosineSimilarityCritical(
        _env, _class, one_len, one, two_len, two,
    );
}

/*
 * Class:     com_github_eliak_VScoreNative
 * Method:    cosineSimilarity2
 * Signature: (I[FI[F)F
 * для работы этой функции JVM должна быть запущена с опцией: -Xcomp
 */
#[no_mangle]
pub extern "system" fn JavaCritical_com_github_eliak_VScoreNative_cosineSimilarity2(
    one_len: jint,
    one_ptr: &jfloat,
    two_len: jint,
    two_ptr: &jfloat,
) -> f32 {
    println!("JavaCritical_");
    let one_slice =
        unsafe { std::slice::from_raw_parts(one_ptr as *const _ as *const f32, one_len as usize) };
    let two_slice =
        unsafe { std::slice::from_raw_parts(two_ptr as *const _ as *const f32, two_len as usize) };
    let similarity = unaligned::cosine_similarity(one_slice, two_slice);
    std::mem::forget(one_slice);
    std::mem::forget(two_slice);
    return similarity;
}

/*
 * Class:     com_github_eliak_VScoreNative
 * Method:    cosineSimilarityCritical
 * Signature: (I[FI[F)F
 */
#[no_mangle]
pub extern "system" fn Java_com_github_eliak_VScoreNative_cosineSimilarityCritical(
    _env: JNIEnv,
    _class: JClass,
    one_len: jint,
    one: jfloatArray,
    two_len: jint,
    two: jfloatArray,
) -> f32 {
    assert_eq!(one_len, two_len);

    let len = one_len as usize;

    let one_auto = _env
        .get_auto_primitive_array_critical(one, ReleaseMode::NoCopyBack)
        .unwrap();
    let one_ptr = one_auto.as_ptr() as *mut f32;
    let one_slice = unsafe { std::slice::from_raw_parts(one_ptr, len) };

    let two_auto = _env
        .get_auto_primitive_array_critical(two, ReleaseMode::NoCopyBack)
        .unwrap();
    let two_ptr = two_auto.as_ptr() as *mut f32;
    let two_slice = unsafe { std::slice::from_raw_parts(two_ptr, len) };

    let similarity = unaligned::cosine_similarity(one_slice, two_slice);

    std::mem::forget(one_slice);
    std::mem::forget(two_slice);

    return similarity;
}

/*
 * Class:     com_github_eliak_VScoreNative
 * Method:    cosineSimilarityCritical
 * Signature: (I[FI[F)F
 */
// #[no_mangle]
// pub extern "system" fn JavaCritical_com_github_eliak_VScoreNative_cosineSimilarityCritical(
//     one_len: jint, one_ptr: &jfloat,
//     two_len: jint, two_ptr: &jfloat
// ) -> f32 {
//     println!("one_len: {:?}", one_len);
//     println!("one_ptr: {:?}", *one_ptr);
//     println!("two_len {:?}", two_len);
//     println!("two_ptr {:?}", two_ptr);
//     // let one_slice = unsafe { std::slice::from_raw_parts(one_ptr as *const _ as *const f32, one_len as usize) };
//     // let two_slice = unsafe { std::slice::from_raw_parts(two_ptr as *const _ as *const f32, two_len as usize) };
//     // println!("one: {:?}\ntwo {:?}", one_slice, two_slice);
//     // let similarity = cosine_similarity(one_slice, two_slice);
//     // println!("similarity {:?}", similarity);
//     // std::mem::forget(one_slice);
//     // std::mem::forget(two_slice);
//     // return similarity;
//     return 0f32;
// }

/*
 * Class:     com_github_eliak_VScoreNative
 * Method:    createScorerFactory
 * Signature: ()J
 */
#[no_mangle]
pub unsafe extern "system" fn Java_com_github_eliak_VScoreNative_createScorerFactory(
    _env: JNIEnv,
    _class: JClass,
) -> i64 {
    let factory = aligned::ScorerFactory::new();
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
pub extern "system" fn Java_com_github_eliak_VScoreNative_destroyScorerFactory(
    _env: JNIEnv,
    _class: JClass,
    factory_ptr: jlong,
) {
    // println!("destroyScorerFactory: {}", factory_ptr);
    let _boxed_factory = unsafe { Box::from_raw(factory_ptr as *mut aligned::ScorerFactory) };
    drop(_boxed_factory);
}

/*
 * Class:     com_github_eliak_VScoreNative
 * Method:    createScorer
 * Signature: (J[F)J
 */
#[no_mangle]
pub unsafe extern "system" fn Java_com_github_eliak_VScoreNative_createScorer(
    _env: JNIEnv,
    _class: JClass,
    factory_ptr: jlong,
    query_vector: jfloatArray,
) -> jlong {
    let factory = &*(factory_ptr as *const aligned::ScorerFactory);
    let scorer = factory.scorer(aligned::Item::from_jni_float_array(&_env, query_vector));
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
    _env: JNIEnv,
    _class: JClass,
    scorer_ptr: jlong,
) {
    // println!("destroyScorer: {}", scorer_ptr);
    let _boxed_scorer = Box::from_raw(scorer_ptr as *mut aligned::Scorer);
    drop(_boxed_scorer);
}

/*
 * Class:     com_github_eliak_VScoreNative
 * Method:    score
 * Signature: (JILcom/github/eliak/VScoreNative/ScorerCallback;)F
 */
#[no_mangle]
pub unsafe extern "system" fn Java_com_github_eliak_VScoreNative_score(
    _env: JNIEnv,
    _class: JClass,
    scorer_ptr: jlong,
    doc_id: jint,
    callback: JObject,
) -> f32 {
    let scorer = &*(scorer_ptr as *const aligned::Scorer);
    scorer.score(&_env, doc_id as aligned::DocId, callback)
}

/*
 * Class:     com_github_eliak_VScoreNative
 * Method:    identity
 * Signature: (F)F
 */
#[no_mangle]
pub unsafe extern "system" fn Java_com_github_eliak_VScoreNative_identity(
    _env: JNIEnv,
    _class: JClass,
    num: jfloat,
) -> f32 {
    num.clone()
}

/****************************************************************************************************
package com.iqmen.iqfacescore;
public class NativeScorerFactory {
    public static native long createScorerFactory();
    public static native long destroyScorerFactory(long factoryPtr);
    public static native long createScorer(long factoryPtr, float[] vector);
    public static native void destroyScorer(long scorerPtr);
    public static native float dotProduct(long scorerPtr, long docID, ScorerCallback callback);
    public static native float cosineSimilarity(long scorerPtr, long docID, ScorerCallback callback);
    public static native long createItem(float[] vector);
    public static native void destroyItem(long ptr);
    public static native float itemDotProduct(long itemPtr1, long itemPtr2);
    public static native float itemDotProductWithVector(long itemPtr, float[] vector);
    public static native float itemCosineSimilarity(long itemPtr1, long itemPtr2);
    public static native float dotProductVectorAndSerializedVector(float[] vector1, byte[] vector2);
}
****************************************************************************************************/
/*
 * Class:     com_iqmen_iqfacescore_NativeScorerFactory
 * Method:    createScorerFactory
 * Signature: ()J
 */
#[no_mangle]
pub unsafe extern "system" fn Java_com_iqmen_iqfacescore_NativeScorerFactory_createScorerFactory(
    _env: JNIEnv,
    _class: JClass,
) -> i64 {
    let factory = aligned::ScorerFactory::new();
    let result = Box::into_raw(Box::new(factory)) as jlong;
    //println!("create scorer factory: {:?}", result);
    result
}

/*
 * Class:     com_iqmen_iqfacescore_NativeScorerFactory
 * Method:    destroyScorerFactory
 * Signature: (J)J
 */
#[no_mangle]
pub extern "system" fn Java_com_iqmen_iqfacescore_NativeScorerFactory_destroyScorerFactory(
    _env: JNIEnv,
    _class: JClass,
    factory_ptr: jlong,
) {
    println!("drop scorer factory: {:?}", factory_ptr);
    let _boxed_factory = unsafe { Box::from_raw(factory_ptr as *mut aligned::ScorerFactory) };
    drop(_boxed_factory);
}

/*
 * Class:     com_iqmen_iqfacescore_NativeScorerFactory
 * Method:    createScorer
 * Signature: (J[F)J
 */
#[no_mangle]
pub unsafe extern "system" fn Java_com_iqmen_iqfacescore_NativeScorerFactory_createScorer(
    _env: JNIEnv,
    _class: JClass,
    factory_ptr: jlong,
    query_vector: jfloatArray,
) -> jlong {
    let factory = &*(factory_ptr as *const aligned::ScorerFactory);
    let scorer = factory.scorer(aligned::Item::from_jni_float_array(&_env, query_vector));
    let result = Box::into_raw(Box::new(scorer)) as jlong;
    //println!("create scorer {:?} by factory: {:?}", result, factory_ptr);
    result
}

/*
 * Class:     com_iqmen_iqfacescore_NativeScorerFactory
 * Method:    destroyScorer
 * Signature: (J)V
 */
#[no_mangle]
pub extern "system" fn Java_com_iqmen_iqfacescore_NativeScorerFactory_destroyScorer(
    _env: JNIEnv,
    _class: JClass,
    scorer_ptr: jlong,
) {
    //println!("drop scorer: {:?}", scorer_ptr);
    let _boxed_scorer = unsafe { Box::from_raw(scorer_ptr as *mut aligned::Scorer) };
    drop(_boxed_scorer);
}

/*
 * Class:     com_iqmen_iqfacescore_NativeScorerFactory
 * Method:    dotProduct
 * Signature: (JILcom/github/eliak/VScoreNative/ScorerCallback;)F
 */
#[no_mangle]
pub unsafe extern "system" fn Java_com_iqmen_iqfacescore_NativeScorerFactory_dotProduct(
    _env: JNIEnv,
    _class: JClass,
    scorer_ptr: jlong,
    doc_id: jlong,
    callback: JObject,
) -> f32 {
    let scorer = &*(scorer_ptr as *const aligned::Scorer);
    scorer.dot_product(&_env, doc_id as aligned::DocId, callback)
}

/*
 * Class:     com_iqmen_iqfacescore_NativeScorerFactory
 * Method:    cosineSimilarity
 * Signature: (JILcom/github/eliak/VScoreNative/ScorerCallback;)F
 */
#[no_mangle]
pub unsafe extern "system" fn Java_com_iqmen_iqfacescore_NativeScorerFactory_cosineSimilarity(
    _env: JNIEnv,
    _class: JClass,
    scorer_ptr: jlong,
    doc_id: jlong,
    callback: JObject,
) -> f32 {
    let scorer = &*(scorer_ptr as *const aligned::Scorer);
    scorer.cosine_similarity(&_env, doc_id as aligned::DocId, callback)
}

/*
 * Class:     com_iqmen_iqfacescore_NativeScorerFactory
 * Method:    createItem
 * Signature: ([F)J
 */
#[no_mangle]
pub unsafe extern "system" fn Java_com_iqmen_iqfacescore_NativeScorerFactory_createItem(
    _env: JNIEnv,
    _class: JClass,
    query_vector: jfloatArray,
) -> jlong {
    let boxed_item = Box::new(aligned::Item::from_jni_float_array(&_env, query_vector));
    let result = Box::into_raw(boxed_item) as jlong;
    //println!("create item {:?} by factory: {:?}", result, factory_ptr);
    result
}

/*
 * Class:     com_iqmen_iqfacescore_NativeScorerFactory
 * Method:    destroyItem
 * Signature: (J)V
 */
#[no_mangle]
pub extern "system" fn Java_com_iqmen_iqfacescore_NativeScorerFactory_destroyItem(
    _env: JNIEnv,
    _class: JClass,
    item_ptr: jlong,
) {
    //println!("drop scorer: {:?}", scorer_ptr);
    let _boxed_item = unsafe { Box::from_raw(item_ptr as *mut aligned::Item) };
    drop(_boxed_item);
}

/*
 * Class:     com_iqmen_iqfacescore_NativeScorerFactory
 * Method:    itemDotProduct
 * Signature: (JJ)F
 */
#[no_mangle]
pub unsafe extern "system" fn Java_com_iqmen_iqfacescore_NativeScorerFactory_itemDotProduct(
    _env: JNIEnv,
    _class: JClass,
    item_ptr_1: jlong,
    item_ptr_2: jlong,
) -> f32 {
    let item_1 = &*(item_ptr_1 as *const aligned::Item);
    let item_2 = &*(item_ptr_2 as *const aligned::Item);
    item_1.dot_product(item_2)
}

/*
 * Class:     com_iqmen_iqfacescore_NativeScorerFactory
 * Method:    itemDotProductWithVector
 * Signature: (J[F)F
 */
#[no_mangle]
pub unsafe extern "system" fn Java_com_iqmen_iqfacescore_NativeScorerFactory_itemDotProductWithVector(
    _env: JNIEnv,
    _class: JClass,
    item_ptr: jlong,
    vector: jfloatArray,
) -> f32 {
    let item = &*(item_ptr as *const aligned::Item);
    let len = _env.get_array_length(vector).unwrap() as usize;
    let vector_auto = _env
        .get_auto_primitive_array_critical(vector, ReleaseMode::NoCopyBack)
        .unwrap();
    let vector_ptr = vector_auto.as_ptr() as *mut f32;
    let vector_slice = unsafe { std::slice::from_raw_parts(vector_ptr, len) };
    let similarity = item.dot_product_with_unaligned(vector_slice);
    std::mem::forget(vector_slice);
    return similarity;
}

/*
 * Class:     com_iqmen_iqfacescore_NativeScorerFactory
 * Method:    itemDotProductWithVector
 * Signature: (JI[F)F
 */
#[no_mangle]
pub unsafe extern "system" fn JavaCritical_com_iqmen_iqfacescore_NativeScorerFactory_itemDotProductWithVector(
    item_ptr: jlong,
    vector_len: jint,
    vector_ptr: &jfloat,
) -> f32 {
    let item = &*(item_ptr as *const aligned::Item);
    let slice = unsafe {
        std::slice::from_raw_parts(vector_ptr as *const _ as *const f32, vector_len as usize)
    };
    let dot_product = item.dot_product_with_unaligned(slice);
    std::mem::forget(slice);
    return dot_product;
}

/*
 * Class:     com_iqmen_iqfacescore_NativeScorerFactory
 * Method:    itemCosineSimilarity
 * Signature: (JJ)F
 */
#[no_mangle]
pub unsafe extern "system" fn Java_com_iqmen_iqfacescore_NativeScorerFactory_itemCosineSimilarity(
    _env: JNIEnv,
    _class: JClass,
    item_ptr_1: jlong,
    item_ptr_2: jlong,
) -> f32 {
    let item_1 = &*(item_ptr_1 as *const aligned::Item);
    let item_2 = &*(item_ptr_2 as *const aligned::Item);
    item_1.cosine_similarity(item_2)
}

/*
 * Class:     com_iqmen_iqfacescore_NativeScorerFactory
 * Method:    dotProductVectors
 * Signature: ([F[F)F
 */
#[no_mangle]
pub extern "system" fn Java_com_iqmen_iqfacescore_NativeScorerFactory_dotProductVectors(
    _env: JNIEnv,
    _class: JClass,
    one: jfloatArray,
    two: jfloatArray,
) -> f32 {
    let one_len = _env.get_array_length(one).unwrap();
    let two_len = _env.get_array_length(two).unwrap();

    assert_eq!(one_len, two_len);

    let one_auto = _env
        .get_auto_primitive_array_critical(one, ReleaseMode::NoCopyBack)
        .unwrap();
    let one_ptr = one_auto.as_ptr() as *mut f32;
    let one_slice = unsafe { std::slice::from_raw_parts(one_ptr, one_len as usize) };

    let two_auto = _env
        .get_auto_primitive_array_critical(two, ReleaseMode::NoCopyBack)
        .unwrap();
    let two_ptr = two_auto.as_ptr() as *mut f32;
    let two_slice = unsafe { std::slice::from_raw_parts(two_ptr, two_len as usize) };

    let similarity = unaligned::dot_prod(one_slice, two_slice);

    std::mem::forget(one_slice);
    std::mem::forget(two_slice);

    return similarity;
}

/*
 * Class:     com_iqmen_iqfacescore_NativeScorerFactory
 * Method:    dotProductVectors
 * Signature: (I[FI[F)F
 * для работы этой функции JVM должна быть запущена с опцией: -Xcomp
 */
#[no_mangle]
pub extern "system" fn JavaCritical_com_iqmen_iqfacescore_NativeScorerFactory_dotProductVectors(
    one_len: jint,
    one_ptr: &jfloat,
    two_len: jint,
    two_ptr: &jfloat,
) -> f32 {
    let one_slice =
        unsafe { std::slice::from_raw_parts(one_ptr as *const _ as *const f32, one_len as usize) };
    let two_slice =
        unsafe { std::slice::from_raw_parts(two_ptr as *const _ as *const f32, two_len as usize) };
    let similarity = unaligned::dot_prod(one_slice, two_slice);
    std::mem::forget(one_slice);
    std::mem::forget(two_slice);
    return similarity;
}

/*
 * Class:     com_iqmen_iqfacescore_NativeScorerFactory
 * Method:    dotProductVectorAndSerializedVector
 * Signature: ([F[B)F
 */
#[no_mangle]
pub extern "system" fn Java_com_iqmen_iqfacescore_NativeScorerFactory_dotProductVectorAndSerializedVector(
    _env: JNIEnv,
    _class: JClass,
    one: jfloatArray,
    two: jbyteArray,
) -> f32 {
    let one_len = _env.get_array_length(one).unwrap();
    let two_len = _env.get_array_length(two).unwrap();

    assert_eq!(two_len % 4, 0);
    assert_eq!(one_len, two_len / 4);

    let one_auto = _env
        .get_auto_primitive_array_critical(one, ReleaseMode::NoCopyBack)
        .unwrap();
    let one_ptr = one_auto.as_ptr() as *mut f32;
    let one_slice = unsafe { std::slice::from_raw_parts(one_ptr, one_len as usize) };

    let two_auto = _env
        .get_auto_primitive_array_critical(two, ReleaseMode::NoCopyBack)
        .unwrap();
    let two_ptr = two_auto.as_ptr() as *mut f32;
    let two_slice = unsafe { std::slice::from_raw_parts(two_ptr, (two_len / 4) as usize) };

    let similarity = unaligned::dot_prod(one_slice, two_slice);

    std::mem::forget(one_slice);
    std::mem::forget(two_slice);

    return similarity;
}

/*
 * Class:     com_iqmen_iqfacescore_NativeScorerFactory
 * Method:    dotProductVectorAndSerializedVector
 * Signature: (I[FI[B)F
 * для работы этой функции JVM должна быть запущена с опцией: -Xcomp
 */
#[no_mangle]
pub extern "system" fn JavaCritical_com_iqmen_iqfacescore_NativeScorerFactory_dotProductVectorAndSerializedVector(
    one_len: jint,
    one_ptr: &jfloat,
    two_len: jint,
    two_ptr: &jbyte,
) -> f32 {
    assert_eq!(two_len % 4, 0);
    assert_eq!(one_len, two_len / 4);

    let one_slice =
        unsafe { std::slice::from_raw_parts(one_ptr as *const _ as *const f32, one_len as usize) };
    let two_slice = unsafe {
        std::slice::from_raw_parts(two_ptr as *const _ as *const f32, (two_len / 4) as usize)
    };
    let similarity = unaligned::dot_prod(one_slice, two_slice);
    std::mem::forget(one_slice);
    std::mem::forget(two_slice);
    return similarity;
}
