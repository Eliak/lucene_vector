package com.github.eliak;

import org.apache.lucene.index.BinaryDocValues;
import org.apache.lucene.index.DocValues;
import org.apache.lucene.index.LeafReaderContext;
import org.apache.lucene.index.Term;
import org.apache.lucene.search.Explanation;
import org.apache.lucene.search.Scorer;
import org.apache.lucene.search.Weight;

import java.io.IOException;
import java.util.LinkedList;
import java.util.List;
import java.util.Set;

public class VWeight extends Weight implements AutoCloseable {

    protected final List<Scorer> scorers = new LinkedList<>();

    /**
     * Sole constructor, typically invoked by sub-classes.
     *
     * @param query the parent query
     */
    protected VWeight(VQuery query) {
        super(query);
    }

    @Override
    public void extractTerms(Set<Term> terms) {

    }

    @Override
    public Explanation explain(LeafReaderContext context, int doc) throws IOException {
        return null;
    }

    @Override
    public final Scorer scorer(LeafReaderContext context) throws IOException {
        final Scorer scorer = scorerInner(context);
        if(scorer == null) {
            return null;
        }
        scorers.add(scorer);
        return scorer;
    }

    protected Scorer scorerInner(LeafReaderContext context) throws IOException {
        final VQuery query = (VQuery) this.parentQuery;
        final BinaryDocValues docValues = context.reader().getBinaryDocValues(query.field);
        if (docValues == null) {
            return null;
        }
        return query.scorerFactory.create(this, docValues, context.docBase);
    }

    @Override
    public boolean isCacheable(LeafReaderContext ctx) {
        return DocValues.isCacheable(ctx, ((VQuery)parentQuery).field);
    }

    @Override
    public void close() {
        for (Scorer scorer : scorers) {
            if(!(scorer instanceof AutoCloseable)) {
                continue;
            }
            try {
                ((AutoCloseable)scorer).close();
            } catch (Exception e) {
                e.printStackTrace();
            }
        }
        scorers.clear();
    }
}
