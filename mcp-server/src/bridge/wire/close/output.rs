use super::*;
use crate::script_close as core;
use serde::ser::{SerializeMap, SerializeSeq};
use serde::{Serialize, Serializer};
macro_rules! fields {($s:ident,$($key:literal=>$value:expr),+ $(,)?)=>{{let mut map=$s.serialize_map(None)?;$(map.serialize_entry($key,&$value)?;)+map.end()}};}
#[derive(Serialize)]
struct Digest<'a> {
    sha256: &'a str,
    utf8_bytes: usize,
}
#[derive(Serialize)]
struct Invalidated<'a> {
    digest: Digest<'a>,
    collection: StampOut<'a>,
    witness: WitnessOut<'a>,
    staleness: StaleOut<'a>,
    reason: ReasonOut,
}
struct SourceSummaryOut<'a>(&'a core::SourceSummary);
impl Serialize for SourceSummaryOut<'_> {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let v = self.0;
        fields!(s,"authority"=>Authority(v.authority),"availability"=>Availability(v.availability),"digest"=>v.digest.as_ref().map(|(h,n)|Digest{sha256:h,utf8_bytes:*n}),"collection"=>v.collection.as_ref().map(stamp_out),"witness"=>v.witness.as_ref().map(witness_out),"staleness"=>v.staleness.as_ref().map(stale_out),"reason"=>v.reason.map(source_reason_out),"invalidated_evidence"=>v.invalidated.as_ref().map(|(h,n,c,w,st,r)|Invalidated{digest:Digest{sha256:h,utf8_bytes:*n},collection:stamp_out(c),witness:witness_out(w),staleness:stale_out(st),reason:source_reason_out(*r)}))
    }
}
#[derive(Serialize)]
struct SourcesOutSummary<'a> {
    #[serde(rename = "D")]
    disk: SourceSummaryOut<'a>,
    #[serde(rename = "R")]
    resource: SourceSummaryOut<'a>,
    #[serde(rename = "B")]
    buffer: SourceSummaryOut<'a>,
}
fn sources(value: &core::SnapshotSummary) -> SourcesOutSummary<'_> {
    SourcesOutSummary {
        disk: SourceSummaryOut(&value.sources[0]),
        resource: SourceSummaryOut(&value.sources[1]),
        buffer: SourceSummaryOut(&value.sources[2]),
    }
}
struct SnapshotOutSummary<'a>(&'a core::SnapshotSummary);
impl Serialize for SnapshotOutSummary<'_> {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let v = self.0;
        let c = v.comparisons;
        let consistency = &v.consistency;
        fields!(s,"target"=>target_out(&v.target),"document"=>document_out(&v.document),"sources"=>sources(v),"dirty"=>dirty_out(&v.dirty),"comparisons"=>ComparisonsOut{disk_resource:Comparison(c.disk_resource()),disk_buffer:Comparison(c.disk_buffer()),resource_buffer:Comparison(c.resource_buffer())},"agreement"=>Agreement(v.agreement),"consistency"=>ConsistencyOut{atomic:consistency.atomic(),stability:Stability(consistency.stability()),checks:Checks(consistency.checks()),detected_changes:ChangesOut(consistency.detected_changes()),recheck_reason:consistency.recheck_reason().map(RecheckReason)},"diagnostics"=>DiagnosticsOut(&v.diagnostics))
    }
}
struct StepOut<'a>(&'a core::Step);
impl Serialize for StepOut<'_> {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let v = self.0;
        fields!(s,"state"=>v.state,"reason"=>v.reason,"collection"=>v.collection.as_ref().map(stamp_out))
    }
}
struct ProgressOut<'a>(&'a core::Progress);
impl Serialize for ProgressOut<'_> {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let v = self.0;
        fields!(s,"closing"=>StepOut(&v.closing),"native_revalidation"=>StepOut(&v.native_revalidation),"verification"=>StepOut(&v.verification))
    }
}
struct ResourceOut<'a>(&'a core::ResourceState);
impl Serialize for ResourceOut<'_> {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let v = self.0;
        fields!(s,"state"=>v.state,"resource_edited"=>v.resource_edited,"reason"=>v.reason,"collection"=>v.collection.as_ref().map(stamp_out))
    }
}
struct ProtectionOut<'a>(&'a core::Protection);
impl Serialize for ProtectionOut<'_> {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let v = self.0;
        fields!(s,"status"=>v.status,"revalidation"=>v.revalidation,"required_count"=>v.required_count,"completed_count"=>v.completed_count,"reason"=>v.reason)
    }
}
struct SelectionOut<'a>(&'a core::Selection);
impl Serialize for SelectionOut<'_> {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let v = self.0;
        fields!(s,"before"=>v.before,"after"=>v.after,"request_effect"=>v.request_effect)
    }
}
struct HistoryOut<'a>(&'a core::History);
impl Serialize for HistoryOut<'_> {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let v = self.0;
        fields!(s,"participation"=>"not_participated","target_buffer"=>v.target_buffer,"unrelated"=>v.unrelated,"reason"=>v.reason)
    }
}
fn interval(v: &ObservationInterval) -> IntervalOut {
    IntervalOut {
        started_unix_ms: v.started_unix_ms(),
        finished_unix_ms: v.finished_unix_ms(),
        elapsed_us: v.elapsed_us(),
    }
}
struct ObservationOut<'a>(&'a core::ObservationSummary);
impl Serialize for ObservationOut<'_> {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let v = self.0;
        fields!(s,"purpose"=>v.purpose,"interval"=>interval(&v.interval),"snapshot"=>SnapshotOutSummary(&v.snapshot))
    }
}
#[derive(Serialize)]
struct EditedOut<'a> {
    availability: &'static str,
    value: Option<bool>,
    collection: Option<StampOut<'a>>,
    reason: Option<&'static str>,
}
struct BeforeOut<'a>(&'a core::Before);
impl Serialize for BeforeOut<'_> {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let v = self.0;
        fields!(s,"document"=>document_out(&v.snapshot.document),"sources"=>sources(&v.snapshot),"dirty"=>dirty_out(&v.snapshot.dirty),"resource_edited"=>EditedOut{availability:if v.resource_edited.is_some(){"observed"}else{"unavailable"},value:v.resource_edited,collection:v.edited_collection.as_ref().map(stamp_out),reason:if v.resource_edited.is_some(){None}else{Some("evidence_unavailable")}},"current_version"=>v.current_version.as_ref().map(DecimalCounter::as_str),"saved_version"=>v.saved_version.as_ref().map(DecimalCounter::as_str),"validity"=>fact_out(v.snapshot.document.validity(),Validity))
    }
}
#[derive(Serialize)]
struct Requested<'a> {
    project_root: &'a str,
    session_id: Option<&'a str>,
    script_path: &'a str,
}
#[derive(Serialize)]
struct DiagnosticOut {
    stage: &'static str,
    reason: &'static str,
    code: Option<i64>,
}
struct DiagnosticList<'a>(&'a [(&'static str, &'static str, Option<i64>)]);
impl Serialize for DiagnosticList<'_> {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let mut sequence = s.serialize_seq(Some(self.0.len()))?;
        for (stage, reason, code) in self.0 {
            sequence.serialize_element(&DiagnosticOut {
                stage,
                reason,
                code: *code,
            })?;
        }
        sequence.end()
    }
}
struct ExpectedOut<'a>(&'a core::Expected);
struct ProvenanceOut<'a>(&'a crate::script_edit::ExpectedRevisionBasis);
impl Serialize for ProvenanceOut<'_> {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let v = self.0;
        let w = v.witnesses();
        let c = v.collections();
        fields!(s,"request_id"=>v.prior_request_id().as_str(),"interval"=>interval(v.prior_interval()),"witnesses"=>[witness_out(&w[0]),witness_out(&w[1]),witness_out(&w[2])],"collections"=>[stamp_out(&c[0]),stamp_out(&c[1]),stamp_out(&c[2])],"document"=>document_out(v.prior_document()),"dirty"=>dirty_out(v.prior_dirty()))
    }
}
impl Serialize for ExpectedOut<'_> {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let v = &self.0.basis;
        let mut hash = String::with_capacity(64);
        const HEX: &[u8; 16] = b"0123456789abcdef";
        for byte in v.source().sha256 {
            hash.push(HEX[(byte >> 4) as usize] as char);
            hash.push(HEX[(byte & 15) as usize] as char);
        }
        fields!(s,"use"=>self.0.use_,"target"=>target_out(v.target()),"document"=>identity_out(v.document()),"source"=>Digest{sha256:&hash,utf8_bytes:v.source().utf8_bytes},"current_version"=>v.current_version().as_str(),"provenance"=>ProvenanceOut(v))
    }
}
struct Output<'a>(&'a ClosingOutcome);
impl Serialize for Output<'_> {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let v = self.0;
        fields!(s,"schema_version"=>1,"operation"=>"close_gdscript","request_id"=>v.request_id.as_str(),"requested_target"=>v.request.as_ref().map(|r|Requested{project_root:r.project_root().as_str(),session_id:r.session_id().map(SessionId::as_str),script_path:r.script_path().as_str()}),"resolved_target"=>v.target.as_ref().map(target_out),"interval"=>interval(&v.interval),"outcome"=>v.kind,"reason"=>v.reason,"stage"=>v.stage,"application"=>v.application,"expected"=>v.expected.as_ref().map(ExpectedOut),"progress"=>ProgressOut(&v.progress),"before"=>v.before.as_ref().map(BeforeOut),"observation"=>v.observation.as_ref().map(ObservationOut),"resource_state"=>ResourceOut(&v.resource),"protection"=>ProtectionOut(&v.protection),"selection"=>v.selection.as_ref().map(SelectionOut),"history"=>HistoryOut(&v.history),"diagnostics"=>DiagnosticList(&v.diagnostics),"safe_next_action"=>"Obtain a fresh ordinary observation of the explicit original target before another intentional action")
    }
}
pub(super) fn encode(value: &ClosingOutcome) -> Result<Vec<u8>, RoutingFailure> {
    let bytes = serde_json::to_vec(&Output(value)).map_err(|_| bad(Stage::Finalize))?;
    if bytes.len() > RESULT_LIMIT {
        return Err(bad(Stage::Finalize));
    }
    Ok(bytes)
}
