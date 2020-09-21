package com.github.eliak;

import org.testng.annotations.Test;

import java.io.IOException;

import static com.github.eliak.ScoreUtils.*;
import static org.testng.Assert.*;

public class VScoreNativeTest {
    @Test
    public void cosineSimilarityCritical2() {
        final int size = 16;
        final float[] one = generateArray(size, true);
        final float[] two = generateArray(size, true);
        final float similarity = VScoreNative.cosineSimilarity2(one, two);
        final float similarity_2 = VScoreNative.cosineSimilarity(one, two);
        assertEquals(similarity, similarity_2);
    }

    @Test
    public void cosineSimilarityNaive() {
        final int size = 16;
        final float[] one = generateArray(size, true);
        final float[] two = generateArray(size, true);
        final float similarity = VScoreNative.cosineSimilarityCritical(one.length, one, two.length, two);
        final float similarity_2 = VScoreNative.cosineSimilarity(one, two);
        assertEquals(Math.round(similarity * 10000), 10000f);
    }

    @Test
    public void naive() {
        final float[] array = generateArray(16, true);
        final long scorerFactoryPtr = VScoreNative.createScorerFactory();
        final long scorerPtr = VScoreNative.createScorer(scorerFactoryPtr, array);
        final float similarity1 = VScoreNative.score(scorerPtr, 0, () -> array);
        assertEquals(Math.round(similarity1 * 10000), 10000f);
        final float similarity2 = VScoreNative.score(scorerPtr, 0, () -> array);
        assertEquals(Math.round(similarity2 * 10000), 10000f);
        VScoreNative.destroyScorer(scorerPtr);
        VScoreNative.destroyScorerFactory(scorerFactoryPtr);
    }
}