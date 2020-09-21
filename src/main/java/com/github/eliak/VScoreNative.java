package com.github.eliak;

import java.io.IOException;

public class VScoreNative {
    public static native float cosineSimilarity(float[] one, float[] another);
    public static native float cosineSimilarity2(float[] one, float[] another);
    public static native float cosineSimilarityCritical(int one_len, float[] one, int another_len, float[] another);

    public static native long createScorerFactory();
    public static native long destroyScorerFactory(long factoryPtr);
    public static native long createScorer(long factoryPtr, float[] vector);
    public static native void destroyScorer(long scorerPtr);
    public static native float score(long scorerPtr, int docID, ScorerCallback callback);
    public static native float identity(float num);

    static {
        System.load("D:\\Dev\\lucene_vector\\rust\\target\\release\\rust.dll");
    }

    interface ScorerCallback {
        float[] binaryValue() throws IOException;
    }
}
