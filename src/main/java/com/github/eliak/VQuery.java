package com.github.eliak;

import org.apache.lucene.search.IndexSearcher;
import org.apache.lucene.search.Query;
import org.apache.lucene.search.ScoreMode;
import org.apache.lucene.search.Weight;

import java.io.IOException;
import java.util.Arrays;
import java.util.LinkedList;
import java.util.List;
import java.util.Objects;

public class VQuery extends Query implements AutoCloseable {

    protected final String field;
    protected final float[] vector;
    protected final List<Weight> weights = new LinkedList<>();
    protected final VScorerFactory scorerFactory;

    public VQuery(String field, float[] vector, VScorerFactory scorerFactory) {
        this.field = field;
        this.vector = vector;
        this.scorerFactory = scorerFactory;
    }

    @Override
    public Weight createWeight(IndexSearcher searcher, ScoreMode scoreMode, float boost) throws IOException {
        final Weight weight = createWeightInner(searcher, scoreMode, boost);
        if(weight == null) {
            return null;
        }
        weights.add(weight);
        return weight;
    }

    protected Weight createWeightInner(IndexSearcher searcher, ScoreMode scoreMode, float boost) throws IOException {
        return new VWeight(this);
    }

    @Override
    public String toString(String field) {
        return "VQuery [field=field=" + this.field + ']';
    }

    @Override
    public boolean equals(Object o) {
        if (this == o) return true;
        if (o == null || getClass() != o.getClass()) return false;
        VQuery vQuery = (VQuery) o;
        return Objects.equals(field, vQuery.field) &&
                Arrays.equals(vector, vQuery.vector) &&
                Objects.equals(weights, vQuery.weights) &&
                Objects.equals(scorerFactory, vQuery.scorerFactory);
    }

    @Override
    public int hashCode() {
        int result = Objects.hash(field, weights, scorerFactory);
        result = 31 * result + Arrays.hashCode(vector);
        return result;
    }

    @Override
    public void close() {
        for (Weight weight : weights) {
            if(!(weight instanceof AutoCloseable)) {
                continue;
            }
            try {
                ((AutoCloseable)weight).close();
            } catch (Exception e) {
                e.printStackTrace();
            }
        }
        weights.clear();
    }
}
