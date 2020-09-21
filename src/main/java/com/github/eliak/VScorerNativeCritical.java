package com.github.eliak;

import org.apache.lucene.index.BinaryDocValues;

import java.io.IOException;
import java.util.Map;

public class VScorerNativeCritical extends VScorerSimpleCache {
    final float[] queryVectorWithDotProduct;
    public VScorerNativeCritical(VWeight weight, BinaryDocValues docValues, int docBase, Map<Integer, float[]> cache) {
        super(weight, docValues, docBase, cache);
        this.queryVectorWithDotProduct = new float[this.queryVector.length + 1];
        System.arraycopy(this.queryVector, 0, this.queryVectorWithDotProduct, 0, this.queryVector.length);
        this.queryVectorWithDotProduct[this.queryVector.length] = (float) this.queryDotProduct;
    }

    @Override
    public float score() throws IOException {
        final float[] floats = floatValue();
        return VScoreNative.cosineSimilarityCritical(queryVectorWithDotProduct.length, queryVectorWithDotProduct, floats.length, floats);
    }
}
