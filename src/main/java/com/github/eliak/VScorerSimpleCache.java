package com.github.eliak;

import org.apache.lucene.index.BinaryDocValues;
import org.apache.lucene.util.BytesRef;

import java.io.IOException;
import java.nio.ByteBuffer;
import java.util.Map;

public class VScorerSimpleCache extends VScorer {
    protected final Map<Integer, float[]> cache;

    public VScorerSimpleCache(VWeight weight, BinaryDocValues docValues, int docBase, Map<Integer, float[]> cache) {
        super(weight, docValues, docBase);
        this.cache = cache;
    }

    @Override
    public float score() throws IOException {
        float[] floats = floatValue();
        double dotProduct = 0.0;
        for (int i = 0; i < queryVector.length; i++) {
            dotProduct += queryVector[i] * floats[i];
        }
        double vectorMagnitude = floats[queryVector.length];
        return (float) (dotProduct / (queryDotProduct * vectorMagnitude));
    }

    protected float[] floatValue() throws IOException {
        final int docID = docValues.docID() + docBase;
        float[] floats = cache.get(docID);
        if(floats == null) {
            final BytesRef vector = docValues.binaryValue();
            final ByteBuffer byteBuffer = ByteBuffer.wrap(vector.bytes, vector.offset, vector.length);
            floats = new float[vector.length / Float.BYTES];
            for (int i = 0; i < floats.length; i++) {
                floats[i] = byteBuffer.getFloat();
            }
            cache.put(docID, floats);
        }
        return floats;
    }
}
