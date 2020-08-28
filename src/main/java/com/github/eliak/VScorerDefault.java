package com.github.eliak;

import org.apache.lucene.index.BinaryDocValues;
import org.apache.lucene.util.BytesRef;

import java.io.IOException;
import java.nio.ByteBuffer;

public class VScorerDefault extends VScorer {
    /**
     * Constructs a Scorer
     *
     * @param weight    The scorers <code>Weight</code>.
     * @param docValues
     */
    public VScorerDefault(VWeight weight, BinaryDocValues docValues, int docBase) {
        super(weight, docValues, docBase);
    }

    @Override
    public float score() throws IOException {
        final BytesRef vector = docValues.binaryValue();
        final ByteBuffer byteBuffer = ByteBuffer.wrap(vector.bytes, vector.offset, vector.length);
        double dotProduct = 0.0;
        for (float queryValue : queryVector) {
            dotProduct += queryValue * byteBuffer.getFloat();
        }
        double vectorMagnitude = byteBuffer.getFloat();
        return (float) (dotProduct / (queryDotProduct * vectorMagnitude));
    }
}
