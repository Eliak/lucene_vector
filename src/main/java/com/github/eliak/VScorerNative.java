package com.github.eliak;

import org.apache.lucene.index.BinaryDocValues;
import org.apache.lucene.util.BytesRef;

import java.io.IOException;
import java.nio.ByteBuffer;

public class VScorerNative extends VScorer implements VScoreNative.ScorerCallback, AutoCloseable  {
    final float[] queryVectorWithDotProduct;
    final long scorerPtr;
    boolean closed;

    public VScorerNative(VScorerNative sibling, BinaryDocValues docValues, int docBase) {
        super((VWeight) sibling.getWeight(), docValues, docBase);
        this.queryVectorWithDotProduct = sibling.queryVectorWithDotProduct;
        this.scorerPtr = sibling.scorerPtr;
        sibling.closed = true;
    }

    public VScorerNative(VWeight weight, BinaryDocValues docValues, int docBase, long factoryPtr) {
        super(weight, docValues, docBase);
        this.queryVectorWithDotProduct = new float[this.queryVector.length + 1];
        System.arraycopy(this.queryVector, 0, this.queryVectorWithDotProduct, 0, this.queryVector.length);
        this.queryVectorWithDotProduct[this.queryVector.length] = (float) this.queryDotProduct;
        this.scorerPtr = VScoreNative.createScorer(factoryPtr, this.queryVectorWithDotProduct);
    }

    @Override
    public float score() throws IOException {
        final int docID = docValues.docID() + docBase;
        return VScoreNative.score(scorerPtr, docID, this);
    }

    public float[] binaryValue() throws IOException {
        final BytesRef vector = docValues.binaryValue();
        final ByteBuffer byteBuffer = ByteBuffer.wrap(vector.bytes, vector.offset, vector.length);
        final float[] floats = new float[vector.length / Float.BYTES];
        for (int i = 0; i < floats.length; i++) {
            floats[i] = byteBuffer.getFloat();
        }
        return floats;
    }

    @Override
    public void close() {
        if(closed) {
            return;
        }
        try {
            VScoreNative.destroyScorer(scorerPtr);
        } catch (Throwable e) {
            e.printStackTrace();
        } finally {
            closed = true;
        }
    }
}
