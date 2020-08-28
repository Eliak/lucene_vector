package com.github.eliak;

import org.apache.lucene.search.TopDocs;
import org.openjdk.jmh.annotations.Benchmark;
import org.openjdk.jmh.annotations.BenchmarkMode;
import org.openjdk.jmh.annotations.Mode;
import org.openjdk.jmh.infra.Blackhole;

import java.io.IOException;

public class BenchmarkRunner {

    @Benchmark
    @BenchmarkMode(Mode.AverageTime)
    public void benchSearch(BenchSearchState state, Blackhole blackhole) throws IOException {
        final TopDocs search = state.searcher.search(state.query, 100);
        state.query.close();
        blackhole.consume(search.scoreDocs);
    }

    // @Benchmark
    // @BenchmarkMode(Mode.AverageTime)
    public void benchCosineSimilarity(BenchSimilarityState state, Blackhole blackhole) throws IOException {
        for (int i = 0; i < 1000000; i++) {
            blackhole.consume(i + cosineSimilarity(state.vector, state.vector, i));
        }
    }

    public static float cosineSimilarity(float[] one, float[] another, int c) {
        final int size = one.length - 1;
        double dotProduct = c;
        for (int i = 0; i < size; i++) {
            dotProduct += one[i] * another[i];
        }
        return (float) (dotProduct / (one[size] * another[size]));
    }

    public static void main(String[] args) throws Exception {
        org.openjdk.jmh.Main.main(args);
    }
}
