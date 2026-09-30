use super::*;
use crate::script_open::*;
use serde::{Serialize, Serializer};
macro_rules! fields {($s:ident,$($key:literal=>$value:expr),+ $(,)?)=>{{use serde::ser::SerializeMap;let mut map=$s.serialize_map(None)?;$(map.serialize_entry($key,&$value)?;)+map.end()}};}
struct StepOut<'a>(&'a Step);
impl Serialize for StepOut<'_> {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let v = self.0;
        fields!(s,"state"=>v.state.name(),"reason"=>v.reason.map(Reason::name),"collection"=>v.collection.as_ref().map(stamp_out),"script_instance_id"=>v.script.as_ref().map(DecimalCounter::as_str),"editor_instance_id"=>v.editor.as_ref().map(DecimalCounter::as_str),"buffer_instance_id"=>v.buffer.as_ref().map(DecimalCounter::as_str))
    }
}
struct BindingOut<'a>(&'a Progress);
impl Serialize for BindingOut<'_> {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let v = &self.0.binding;
        fields!(s,"state"=>v.state.name(),"reason"=>v.reason.map(Reason::name),"collection"=>v.collection.as_ref().map(stamp_out),"script_instance_id"=>v.script.as_ref().map(DecimalCounter::as_str),"editor_instance_id"=>v.editor.as_ref().map(DecimalCounter::as_str),"buffer_instance_id"=>v.buffer.as_ref().map(DecimalCounter::as_str),"mode"=>self.0.mode)
    }
}
struct ProgressOut<'a>(&'a Progress);
impl Serialize for ProgressOut<'_> {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let v = self.0;
        fields!(s,"resource_binding"=>BindingOut(v),"initial_compilation"=>StepOut(&v.compilation),"document_open"=>StepOut(&v.document),"verification"=>StepOut(&v.verification))
    }
}
#[derive(Serialize)]
struct DigestOut<'a> {
    sha256: &'a str,
    utf8_bytes: usize,
}
struct SummaryOut<'a>(&'a SourceSummary);
#[derive(Serialize)]
struct InvalidSummaryOut<'a> {
    digest: DigestOut<'a>,
    collection: StampOut<'a>,
    witness: WitnessOut<'a>,
    staleness: StaleOut<'a>,
    reason: ReasonOut,
}
impl Serialize for SummaryOut<'_> {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let v = self.0;
        fields!(s,"authority"=>Authority(v.authority),"availability"=>Availability(v.availability),"digest"=>v.digest.as_ref().map(|(h,n)|DigestOut{sha256:h,utf8_bytes:*n}),"collection"=>v.collection.as_ref().map(stamp_out),"witness"=>v.witness.as_ref().map(witness_out),"staleness"=>v.staleness.as_ref().map(stale_out),"reason"=>v.reason.map(source_reason_out),"invalidated_evidence"=>v.invalidated.as_ref().map(|(h,n,c,w,st,r)|InvalidSummaryOut{digest:DigestOut{sha256:h,utf8_bytes:*n},collection:stamp_out(c),witness:witness_out(w),staleness:stale_out(st),reason:source_reason_out(*r)}))
    }
}
struct EditedOut<'a>(&'a ResourceEdited);
impl Serialize for EditedOut<'_> {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let v = self.0;
        fields!(s,"availability"=>Availability(v.availability),"value"=>v.value,"collection"=>v.collection.as_ref().map(stamp_out),"witness"=>v.witness.as_ref().map(witness_out),"reason"=>v.reason.map(Reason::name))
    }
}
struct BeforeOut<'a>(&'a Before);
impl Serialize for BeforeOut<'_> {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let v = self.0;
        #[derive(Serialize)]
        struct SourceRecords<'a> {
            #[serde(rename = "D")]
            d: SummaryOut<'a>,
            #[serde(rename = "R")]
            r: SummaryOut<'a>,
            #[serde(rename = "B")]
            b: SummaryOut<'a>,
        }
        fields!(s,"mode"=>v.mode,"document"=>document_out(&v.document),"sources"=>SourceRecords{d:SummaryOut(&v.sources[0]),r:SummaryOut(&v.sources[1]),b:SummaryOut(&v.sources[2])},"dirty"=>dirty_out(&v.dirty),"resource_edited"=>EditedOut(&v.resource_edited))
    }
}
#[derive(Serialize)]
struct OpeningSnapshotOut<'a> {
    #[serde(flatten)]
    snapshot: SnapshotOut<'a>,
    interval: IntervalOut,
}
struct ObservationOut<'a>(&'a script_open::Observation);
impl Serialize for ObservationOut<'_> {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let v = self.0;
        fields!(s,"purpose"=>v.purpose,"snapshot"=>OpeningSnapshotOut{snapshot:snapshot_out(&v.snapshot),interval:IntervalOut{started_unix_ms:v.interval.started_unix_ms(),finished_unix_ms:v.interval.finished_unix_ms(),elapsed_us:v.interval.elapsed_us()}})
    }
}
#[derive(Serialize)]
struct Requested<'a> {
    project_root: &'a str,
    session_id: Option<&'a str>,
    script_path: &'a str,
}
#[derive(Serialize)]
struct Disposition {
    status: &'static str,
    reason: Option<&'static str>,
}
#[derive(Serialize)]
struct HistoryOut {
    participation: &'static str,
    preservation: &'static str,
    reason: Option<&'static str>,
}
#[derive(Serialize)]
struct SelectionOut {
    before: &'static str,
    after: &'static str,
    request_effect: &'static str,
}
#[derive(Serialize)]
struct ParseOut<'a> {
    state: &'static str,
    origin: Option<&'static str>,
    code: Option<i64>,
    collection: Option<StampOut<'a>>,
    diagnostics: [(); 0],
}
#[derive(Serialize)]
struct DiagnosticOut {
    stage: &'static str,
    reason: &'static str,
    code: Option<i64>,
}
struct DiagnosticsOut<'a>(&'a [OpenDiagnostic]);
impl Serialize for DiagnosticsOut<'_> {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeSeq;
        let mut seq = s.serialize_seq(Some(self.0.len()))?;
        for v in self.0 {
            seq.serialize_element(&DiagnosticOut {
                stage: v.stage.name(),
                reason: v.reason.name(),
                code: v.code,
            })?;
        }
        seq.end()
    }
}
struct Output<'a>(&'a OpeningOutcome);
impl Serialize for Output<'_> {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let v = self.0;
        fields!(s,"schema_version"=>1,"operation"=>"open_gdscript","request_id"=>v.request_id.as_str(),"requested_target"=>v.request.as_ref().map(|r|Requested{project_root:r.project_root().as_str(),session_id:r.session_id().map(SessionId::as_str),script_path:r.script_path().as_str()}),"resolved_target"=>v.target.as_ref().map(target_out),"interval"=>IntervalOut{started_unix_ms:v.interval.started_unix_ms(),finished_unix_ms:v.interval.finished_unix_ms(),elapsed_us:v.interval.elapsed_us()},"outcome"=>v.kind.name(),"reason"=>v.reason.name(),"stage"=>v.stage.name(),"application"=>v.application.name(),"progress"=>ProgressOut(&v.progress),"before"=>v.before.as_ref().map(BeforeOut),"observation"=>v.observation.as_ref().map(ObservationOut),"resource_edited"=>EditedOut(&v.resource_edited),"target_parse"=>ParseOut{state:v.parse.state,origin:if v.parse.collection.is_some(){Some("initial_compilation")}else{None},code:v.parse.code,collection:v.parse.collection.as_ref().map(stamp_out),diagnostics:[]},"protection"=>Disposition{status:v.protection.0,reason:v.protection.1.map(Reason::name)},"history"=>HistoryOut{participation:"not_participated",preservation:v.history.0,reason:v.history.1.map(Reason::name)},"selection"=>v.selection.as_ref().map(|s|SelectionOut{before:s.before,after:s.after,request_effect:s.effect}),"diagnostics"=>DiagnosticsOut(&v.diagnostics),"safe_next_action"=>v.next_action())
    }
}
/// Serialize one bounded result without cloning target source into a JSON tree.
/// # Errors
/// Rejects serialization failures and the fixed result bound. The closed typed
/// output shape itself is shallower than the depth-32 protocol limit.
pub fn encode_outcome(outcome: &OpeningOutcome) -> Result<Vec<u8>, RoutingFailure> {
    let bytes = serde_json::to_vec(&Output(outcome)).map_err(|_| bad(Stage::Finalize))?;
    if bytes.len() > RESULT_LIMIT {
        return Err(bad(Stage::Finalize));
    }
    Ok(bytes)
}
pub fn exit_code(outcome: &OpeningOutcome) -> i32 {
    match outcome.kind {
        Kind::VerifiedNewlyOpened | Kind::AlreadyOpenUnchanged => 0,
        Kind::Refused if outcome.reason == Reason::InvalidRequest => 2,
        Kind::Refused => 3,
        Kind::AppliedUnverified | Kind::EffectsUnknown => 4,
    }
}
/// Structured argument refusal; no checked target or source was established.
pub fn invalid_request(id: RequestId, interval: ObservationInterval) -> OpeningOutcome {
    Attempt::invalid(id, interval)
}
/// Well-formed unsupported target refusal, before any authentication or acquisition.
pub fn invalid_document_kind(
    request: ObservationRequest,
    interval: ObservationInterval,
) -> OpeningOutcome {
    Attempt::invalid_document_kind(request, interval)
}
