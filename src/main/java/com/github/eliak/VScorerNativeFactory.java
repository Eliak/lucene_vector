package com.github.eliak;

import org.apache.lucene.index.BinaryDocValues;
import org.apache.lucene.search.Scorer;

public class VScorerNativeFactory implements AutoCloseable, VScorerFactory {
    protected final long factoryPtr;
    private boolean closed;

    public VScorerNativeFactory() {
        this.factoryPtr = VScoreNative.createScorerFactory();
    }

    @Override
    public void close() {
        if (!closed) {
            closed = true;
            try {
                VScoreNative.destroyScorerFactory(factoryPtr);
            } catch (Throwable e) {
                e.printStackTrace();
            }
        }
    }

    @Override
    public VScorer create(VWeight weight, BinaryDocValues docValues, int docBase) {
        for (Scorer scorer : weight.scorers) {
            if (!(scorer instanceof VScorerNative)) {
                continue;
            }
            if (!((VScorerNative) scorer).closed) {
                return new VScorerNative((VScorerNative) scorer, docValues, docBase);
            }
        }
        return new VScorerNative(weight, docValues, docBase, factoryPtr);
    }
}
