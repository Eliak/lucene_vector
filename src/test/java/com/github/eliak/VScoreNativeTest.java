package com.github.eliak;

import org.testng.annotations.Test;

import java.io.IOException;

import static com.github.eliak.ScoreUtils.*;
import static org.testng.Assert.*;

public class VScoreNativeTest {
    @Test
    public void cosineSimilarityNaive() {
        final float[] array = generateArray(true);
        final float similarity = VScoreNative.cosineSimilarity(array, array);
        assertEquals(Math.round(similarity * 10000), 10000f);
    }
    @Test
    public void naive() {
        final float[] array = generateArray(true);
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