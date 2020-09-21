package com.github.eliak;

import org.openjdk.jmh.annotations.Level;
import org.openjdk.jmh.annotations.Scope;
import org.openjdk.jmh.annotations.Setup;
import org.openjdk.jmh.annotations.State;

import java.io.IOException;

@State(Scope.Benchmark)
public class BenchSimilarityState {
    public float[] vector_1;
    public float[] vector_2;
    @Setup(Level.Trial)
    public void setUp() throws IOException {
        vector_1 = BenchUtils.generateArray(true);
        vector_2 = BenchUtils.generateArray(true);
        System.out.println(">>>>>>>>>>>>>>>>>>");
        System.out.println("------------------" + VScoreNative.cosineSimilarity2(vector_1, vector_2));
        System.out.println("<<<<<<<<<<<<<<<<<<<");
    }
}
