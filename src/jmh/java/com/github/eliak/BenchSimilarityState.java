package com.github.eliak;

import org.openjdk.jmh.annotations.Level;
import org.openjdk.jmh.annotations.Scope;
import org.openjdk.jmh.annotations.Setup;
import org.openjdk.jmh.annotations.State;

import java.io.IOException;

@State(Scope.Benchmark)
public class BenchSimilarityState {
    public float[] vector;
    @Setup(Level.Trial)
    public void setUp() throws IOException {
        vector = BenchUtils.generateArray(true);
    }
}
