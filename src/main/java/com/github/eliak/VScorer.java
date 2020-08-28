package com.github.eliak;

import org.apache.lucene.index.BinaryDocValues;
import org.apache.lucene.search.DocIdSetIterator;
import org.apache.lucene.search.Scorer;

import java.io.IOException;

public abstract class VScorer extends Scorer {

    protected final int docBase;
    protected final float[] queryVector;
    protected final double queryDotProduct;
    protected final BinaryDocValues docValues;

    /**
     * Constructs a Scorer
     * @param weight The scorers <code>Weight</code>.
     * @param docValues
     */
    public VScorer(VWeight weight, BinaryDocValues docValues, int docBase) {
        super(weight);
        this.docValues = docValues;
        this.docBase = docBase;
        this.queryVector = ((VQuery) weight.getQuery()).vector;
        double dotProduct = 0;
        for (float v : queryVector) {
            dotProduct += v * v;
        }
        this.queryDotProduct = Math.sqrt(dotProduct);
    }

    @Override
    public DocIdSetIterator iterator() {
        return docValues;
    }

    @Override
    public float getMaxScore(int upTo) throws IOException {
        return 1;
    }

    @Override
    public int docID() {
        return docValues.docID();
    }

    public static float cosineSimilarity(float[] one, float[] another) {
        final int size = one.length - 1;
        double dotProduct = 0.0;
        for (int i = 0; i < size; i++) {
            dotProduct += one[i] * another[i];
        }
        return (float) (dotProduct / (one[size] * another[size]));
    }
}
