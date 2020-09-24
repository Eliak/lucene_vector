use test::Bencher;

use hashers::fx_hash::FxHasher32;

use rand::Rng;
use std::any::type_name_of_val;
use std::collections::HashMap;
use std::hash::BuildHasherDefault;

use crate::aligned::{Item, ScorerFactory};
use crate::unaligned;
use std::sync::Arc;

fn generate_array(size: usize) -> Vec<f32> {
    let mut vec = Vec::new();
    let mut rng = rand::thread_rng();
    let mut dot_product: f64 = 0 as f64;
    for _ in 0..size {
        let val = rng.gen::<f32>();
        dot_product += (val as f64).powi(2);
        vec.push(val);
    }
    vec.push(dot_product.sqrt() as f32);
    return vec;
}

#[test]
fn test_cosine_similarity_vec() {
    let vec = generate_array(64);
    assert_eq!(
        (unaligned::cosine_similarity(&vec, &vec) * 10000f32).round(),
        10000f32
    );
}

#[test]
fn test_cosine_similarity_item() {
    let item = Item::random();
    // unsafe {
    //     let x1 = item.vector.get_unchecked(0);
    //     let target_ptr = x1 as *const f32;
    //     let i = mem::align_of::<f32x8>();
    //     let i1 = target_ptr.align_offset(i);
    //     assert_eq!(i1, 0);
    // }
    let similarity = item.cosine_similarity(&item);
    assert_eq!((similarity * 10000f32).round(), 10000f32);
}

#[test]
fn test_cosine_similarity_item2() {
    let len = 10000;
    let mut vec = Vec::with_capacity(len);
    for _ in 0..len {
        let item = Item::random();
        vec.push(item);
    }
    let size = 1000000;
    let mut similarity: f32 = 0f32;
    for i in 0..size {
        let one = &vec[i % len];
        let two = &vec[len - 1 - (i % len)];
        similarity += one.cosine_similarity(two);
    }
    println!("{:?}", similarity);
}

/**
* Настоящий бенчмарк выдаёт малопонятный результат типа
*   test tests::bench_cosine_similarity2 ... bench:  65,161,520 ns/iter (+/- 5,057,415)
*   test result: ok. 0 passed; 0 failed; 0 ignored; 1 measured; 2 filtered out
* по этому написал такое:
*/
#[bench]
fn bench_cosine_similarity(b: &mut Bencher) {
    let len = 10000;
    let mut vec = Vec::with_capacity(len);
    for _ in 0..len {
        vec.push(Item::random());
    }
    b.iter(|| {
        let size = test::black_box(1000000);
        let mut similarity: f32 = 0f32;
        for i in 0..size {
            similarity += &vec[i % len].cosine_similarity(&vec[len - 1 - (i % len)]);
        }
        test::black_box(similarity);
    });
}

#[bench]
fn bench_cosine_similarity2(b: &mut Bencher) {
    let len = 10000;
    let mut vec = Vec::with_capacity(len);
    for _ in 0..len {
        vec.push(generate_array(512));
    }
    b.iter(|| {
        let size = test::black_box(1000000);
        let mut similarity: f32 = 0f32;
        for i in 0..size {
            similarity += unaligned::cosine_similarity(&vec[i % len], &vec[len - 1 - (i % len)]);
        }
        test::black_box(similarity);
    });
}

#[bench]
fn bench_scorer_factory_cache(b: &mut Bencher) {
    let factory = ScorerFactory::new();
    {
        let mut guard = factory.cache.write().unwrap();
        for i in 0..100 {
            guard.insert(i.clone(), Arc::new(Item::new()));
        }
    }

    b.iter(|| {
        let size = test::black_box(100000);
        for i in 0..size {
            let guard = factory.cache.read().unwrap();
            if let Some(v) = guard.get(&(i % 100)) {
                test::black_box(v.clone());
            }
        }
    });
}

#[bench]
fn bench_scorer_factory_map(b: &mut Bencher) {
    let mut map =
        HashMap::with_capacity_and_hasher(1000, BuildHasherDefault::<FxHasher32>::default());
    {
        for i in 0..100 {
            map.insert(i.clone(), Arc::new(vec![i as f32]));
        }
    }

    b.iter(|| {
        let size = test::black_box(100000);
        for i in 0..size {
            if let Some(v) = map.get(&(i % 100)) {
                test::black_box(v.clone());
            }
        }
    });
}

#[test]
fn test_scorer_factory_cache() {
    let factory = ScorerFactory::new();
    {
        let mut guard = factory.cache.write().unwrap();
        println!(
            "type_name_of_val(&guard.hasher()) = {}",
            type_name_of_val(&guard.hasher())
        );
        for i in 0..100 {
            guard.insert(i.clone(), Arc::new(Item::new()));
        }
    }
}
