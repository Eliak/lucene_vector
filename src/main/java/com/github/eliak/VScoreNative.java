package com.github.eliak;

import org.apache.lucene.index.BinaryDocValues;
import org.apache.lucene.search.Scorer;

import java.io.IOException;

public class VScoreNative {
    public static native float cosineSimilarity(float[] one, float[] another);

    public static native long createScorerFactory();
    public static native long destroyScorerFactory(long factoryPtr);
    public static native long createScorer(long factoryPtr, float[] vector);
    public static native void destroyScorer(long scorerPtr);
    public static native float score(long scorerPtr, int docID, ScorerCallback callback);

    static {
        System.load("D:\\Dev\\lucene_vector\\rust\\target\\release\\rust.dll");
    }

    public static class ScorerFactory implements AutoCloseable, VScorerFactory {
        protected final long factoryPtr;
        private boolean closed;

        public ScorerFactory() {
            this.factoryPtr = createScorerFactory();
        }

        @Override
        public void close() {
            if(!closed) {
                closed = true;
                try {
                    destroyScorerFactory(factoryPtr);
                } catch (Throwable e) {
                    e.printStackTrace();
                }
            }
        }

        @Override
        public VScorer create(VWeight weight, BinaryDocValues docValues, int docBase) {
            for (Scorer scorer : weight.scorers) {
                if(!(scorer instanceof VScorerNative)) {
                    continue;
                }
                if(!((VScorerNative)scorer).closed) {
                    return new VScorerNative((VScorerNative) scorer, docValues, docBase);
                }
            }
            return new VScorerNative(weight, docValues, docBase, factoryPtr);
        }
    }

    interface ScorerCallback {
        float[] binaryValue() throws IOException;
    }
}
