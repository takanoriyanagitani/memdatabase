use core::cmp::Ordering;
use core::ops::Bound;

use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::time::SystemTime;

use log::{error, warn};

use futures::stream::TryStreamExt;

use tokio::sync::mpsc::{Receiver, Sender};

use tokio_stream::wrappers::ReceiverStream;

use prost_types::Value;

use tonic::{Request, Response, Status};

use crate::value::btree::Val;

use crate::memdatabase::v1::memory_database_service_server::MemoryDatabaseService;

use crate::memdatabase::v1::bound::Bound as IBound;
use crate::memdatabase::v1::Bound as RBound;
use crate::memdatabase::v1::{DelRequest, DelResponse};
use crate::memdatabase::v1::{RangeRequest, RangeResponse};

use crate::memdatabase::v1::{DGetRequest, DGetResponse};
use crate::memdatabase::v1::{DHasRequest, DHasResponse};
use crate::memdatabase::v1::{DSetRequest, DSetResponse};

use crate::memdatabase::v1::{GetRequest, GetResponse};
use crate::memdatabase::v1::{SetRequest, SetResponse};

use crate::memdatabase::v1::{PopRequest, PopResponse};
use crate::memdatabase::v1::{PushRequest, PushResponse};
use crate::memdatabase::v1::{QLenRequest, QLenResponse};

use crate::memdatabase::v1::{SAddRequest, SAddResponse};
use crate::memdatabase::v1::{SDelRequest, SDelResponse};
use crate::memdatabase::v1::{SLenRequest, SLenResponse};

pub const MAX_RANGE_SIZE_DEFAULT: usize = 10;

pub enum Req {
    Del(DelRequest, Sender<Result<DelResponse, Status>>),
    Range(
        RangeRequest,
        Sender<Receiver<Result<RangeResponse, Status>>>,
    ),

    Set(SetRequest, Sender<Result<SetResponse, Status>>),
    Get(GetRequest, Sender<Result<GetResponse, Status>>),

    DGet(DGetRequest, Sender<Result<DGetResponse, Status>>),
    DHas(DHasRequest, Sender<Result<DHasResponse, Status>>),
    DSet(DSetRequest, Sender<Result<DSetResponse, Status>>),

    Pop(PopRequest, Sender<Result<PopResponse, Status>>),
    Push(PushRequest, Sender<Result<PushResponse, Status>>),
    QLen(QLenRequest, Sender<Result<QLenResponse, Status>>),

    SAdd(SAddRequest, Sender<Result<SAddResponse, Status>>),
    SDel(SDelRequest, Sender<Result<SDelResponse, Status>>),
    SLen(SLenRequest, Sender<Result<SLenResponse, Status>>),
}

impl Req {
    pub async fn handle_set(
        kv: &mut BTreeMap<Vec<u8>, Val>,
        req: SetRequest,
        reply: Sender<Result<SetResponse, Status>>,
    ) {
        let key: Vec<u8> = req.key;
        let oval: Option<Value> = req.value;
        let res: Result<SetResponse, Status> = match oval {
            None => Err(Status::invalid_argument("no value specified")),
            Some(val) => {
                kv.insert(key, Val::Var(val));
                Ok(SetResponse {
                    set_time: Some(SystemTime::now().into()),
                })
            }
        };
        match reply.send(res).await {
            Ok(_) => {}
            Err(e) => error!("{e}"),
        }
    }

    pub async fn handle_get(
        kv: &BTreeMap<Vec<u8>, Val>,
        req: GetRequest,
        reply: Sender<Result<GetResponse, Status>>,
    ) {
        let key: Vec<u8> = req.key;
        let oval: Option<&Val> = kv.get(&key);
        let res: Result<GetResponse, Status> = (|| {
            let v: &Val = oval.ok_or_else(|| Status::not_found("no value found"))?;
            let s: &Value = match v {
                Val::Var(s) => Ok(s),
                _ => Err(Status::invalid_argument("invalid type")),
            }?;
            Ok(GetResponse {
                value: Some(s.clone()),
            })
        })();
        match reply.send(res).await {
            Ok(_) => {}
            Err(e) => error!("{e}"),
        }
    }
}

impl Req {
    pub async fn handle_dset(
        kv: &mut BTreeMap<Vec<u8>, Val>,
        req: DSetRequest,
        reply: Sender<Result<DSetResponse, Status>>,
    ) {
        let key: Vec<u8> = req.key;
        let dkey: Vec<u8> = req.dkey;
        let oval: Option<Value> = req.value;

        let v: Val = kv.remove(&key).unwrap_or_else(|| Val::Map(BTreeMap::new()));
        let res: Result<DSetResponse, Status> = (|| {
            let mut m: BTreeMap<Vec<u8>, Value> = match v {
                Val::Map(m) => Ok(m),
                _ => Err(Status::invalid_argument("the key is not a map")),
            }?;
            let val: Value = oval.ok_or_else(|| Status::invalid_argument("the value missing"))?;
            m.insert(dkey, val);
            let cnt: usize = m.len();
            kv.insert(key, Val::Map(m));
            Ok(DSetResponse {
                count: cnt as u64,
                dset_time: Some(SystemTime::now().into()),
            })
        })();

        match reply.send(res).await {
            Ok(_) => {}
            Err(e) => error!("{e}"),
        }
    }

    pub async fn handle_dget(
        kv: &BTreeMap<Vec<u8>, Val>,
        req: DGetRequest,
        reply: Sender<Result<DGetResponse, Status>>,
    ) {
        let key: Vec<u8> = req.key;
        let dkey: Vec<u8> = req.dkey;
        let res: Result<DGetResponse, Status> = (|| {
            let v: &Val = kv
                .get(&key)
                .ok_or_else(|| Status::not_found("no value found"))?;
            let m: &BTreeMap<Vec<u8>, Value> = match v {
                Val::Map(m) => Ok(m),
                _ => Err(Status::invalid_argument("not a map")),
            }?;
            let s: &Value = m
                .get(&dkey)
                .ok_or_else(|| Status::not_found("no value found"))?;
            Ok(DGetResponse {
                value: Some(s.clone()),
            })
        })();
        match reply.send(res).await {
            Ok(_) => {}
            Err(e) => error!("{e}"),
        }
    }

    pub async fn handle_dhas(
        kv: &BTreeMap<Vec<u8>, Val>,
        req: DHasRequest,
        reply: Sender<Result<DHasResponse, Status>>,
    ) {
        let key: Vec<u8> = req.key;
        let dkey: Vec<u8> = req.dkey;
        let res: Result<DHasResponse, Status> = (|| {
            let v: &Val = kv
                .get(&key)
                .ok_or_else(|| Status::not_found("no value found"))?;
            let m: &BTreeMap<Vec<u8>, Value> = match v {
                Val::Map(m) => Ok(m),
                _ => Err(Status::invalid_argument("not a map")),
            }?;
            let found: bool = m.contains_key(&dkey);
            Ok(DHasResponse { found })
        })();
        match reply.send(res).await {
            Ok(_) => {}
            Err(e) => error!("{e}"),
        }
    }
}

impl Req {
    pub async fn handle_pop(
        kv: &mut BTreeMap<Vec<u8>, Val>,
        req: PopRequest,
        reply: Sender<Result<PopResponse, Status>>,
    ) {
        let key: Vec<u8> = req.key;
        let front: bool = req.front;
        let res: Result<PopResponse, Status> = (|| {
            let val: Val = kv
                .remove(&key)
                .ok_or_else(|| Status::not_found("no value found"))?;
            let mut q: VecDeque<Value> = match val {
                Val::Deq(q) => Ok(q),
                _ => Err(Status::invalid_argument("not a queue")),
            }?;
            let ov: Option<Value> = match front {
                true => q.pop_front(),
                false => q.pop_back(),
            };
            let v: Value = ov.ok_or_else(|| Status::not_found("the queue is empty"))?;
            kv.insert(key, Val::Deq(q));
            Ok(PopResponse {
                value: Some(v),
                pop_time: Some(SystemTime::now().into()),
            })
        })();
        match reply.send(res).await {
            Ok(_) => {}
            Err(e) => error!("{e}"),
        }
    }

    pub async fn handle_qlen(
        kv: &BTreeMap<Vec<u8>, Val>,
        req: QLenRequest,
        reply: Sender<Result<QLenResponse, Status>>,
    ) {
        let key: Vec<u8> = req.key;
        let res: Result<QLenResponse, Status> = (|| {
            let v: &Val = kv
                .get(&key)
                .ok_or_else(|| Status::not_found("no queue found"))?;
            let q: &VecDeque<Value> = match v {
                Val::Deq(q) => Ok(q),
                _ => Err(Status::invalid_argument("not a queue")),
            }?;
            let sz: usize = q.len();
            Ok(QLenResponse { count: sz as u64 })
        })();
        match reply.send(res).await {
            Ok(_) => {}
            Err(e) => error!("{e}"),
        }
    }

    pub async fn handle_push(
        kv: &mut BTreeMap<Vec<u8>, Val>,
        req: PushRequest,
        reply: Sender<Result<PushResponse, Status>>,
    ) {
        let key: Vec<u8> = req.key;
        let front: bool = req.front;
        let ov: Option<Value> = req.value;
        let val: Val = kv.remove(&key).unwrap_or_else(|| Val::Deq(VecDeque::new()));
        let res: Result<PushResponse, Status> = (|| {
            let mut q: VecDeque<Value> = match val {
                Val::Deq(q) => Ok(q),
                _ => Err(Status::invalid_argument("not a queue")),
            }?;
            let v: Value = ov.ok_or_else(|| Status::invalid_argument("the value missing"))?;
            match front {
                true => q.push_front(v),
                false => q.push_back(v),
            };
            let sz: usize = q.len();
            kv.insert(key, Val::Deq(q));
            Ok(PushResponse {
                count: sz as u64,
                push_time: Some(SystemTime::now().into()),
            })
        })();
        match reply.send(res).await {
            Ok(_) => {}
            Err(e) => error!("{e}"),
        }
    }
}

impl Req {
    pub async fn handle_sadd(
        kv: &mut BTreeMap<Vec<u8>, Val>,
        req: SAddRequest,
        reply: Sender<Result<SAddResponse, Status>>,
    ) {
        let key: Vec<u8> = req.key;
        let val: Vec<u8> = req.val;
        let v: Val = kv.remove(&key).unwrap_or_else(|| Val::Set(BTreeSet::new()));
        let res: Result<SAddResponse, Status> = (|| {
            let mut s: BTreeSet<Vec<u8>> = match v {
                Val::Set(s) => Ok(s),
                _ => Err(Status::invalid_argument("not a set")),
            }?;
            s.insert(val);
            let sz: usize = s.len();
            kv.insert(key, Val::Set(s));
            Ok(SAddResponse {
                count: sz as u64,
                sadd_time: Some(SystemTime::now().into()),
            })
        })();
        match reply.send(res).await {
            Ok(_) => {}
            Err(e) => error!("{e}"),
        }
    }

    pub async fn handle_sdel(
        kv: &mut BTreeMap<Vec<u8>, Val>,
        req: SDelRequest,
        reply: Sender<Result<SDelResponse, Status>>,
    ) {
        let key: Vec<u8> = req.key;
        let val: Vec<u8> = req.val;
        let res: Result<SDelResponse, Status> = (|| {
            let v: &mut Val = kv
                .get_mut(&key)
                .ok_or_else(|| Status::not_found("no val found"))?;
            let s: &mut BTreeSet<Vec<u8>> = match v {
                Val::Set(s) => Ok(s),
                _ => Err(Status::invalid_argument("not a set")),
            }?;
            s.remove(&val);
            let sz: usize = s.len();
            Ok(SDelResponse {
                count: sz as u64,
                sdel_time: Some(SystemTime::now().into()),
            })
        })();
        match reply.send(res).await {
            Ok(_) => {}
            Err(e) => error!("{e}"),
        }
    }

    pub async fn handle_slen(
        kv: &BTreeMap<Vec<u8>, Val>,
        req: SLenRequest,
        reply: Sender<Result<SLenResponse, Status>>,
    ) {
        let key: Vec<u8> = req.key;
        let res: Result<SLenResponse, Status> = (|| {
            let v: &Val = kv
                .get(&key)
                .ok_or_else(|| Status::not_found("no set found"))?;
            let s: &BTreeSet<Vec<u8>> = match v {
                Val::Set(s) => Ok(s),
                _ => Err(Status::invalid_argument("not a set")),
            }?;
            let sz: usize = s.len();
            Ok(SLenResponse { count: sz as u64 })
        })();
        match reply.send(res).await {
            Ok(_) => {}
            Err(e) => error!("{e}"),
        }
    }
}

pub fn bound_convert(ob: Option<RBound>) -> Result<Bound<Vec<u8>>, Status> {
    let r: RBound = ob.ok_or_else(|| Status::invalid_argument("invalid bound"))?;
    let i: IBound = r
        .bound
        .ok_or_else(|| Status::invalid_argument("invalid bound"))?;
    match i {
        IBound::Included(v) => Ok(Bound::Included(v)),
        IBound::Excluded(v) => Ok(Bound::Excluded(v)),
    }
}

pub fn bound2t<T>(b: &Bound<T>) -> Result<&T, Status> {
    match b {
        Bound::Included(t) => Ok(t),
        Bound::Excluded(t) => Ok(t),
        Bound::Unbounded => Err(Status::invalid_argument("unexpected bount")),
    }
}

pub fn bounds2ord<T>(l: &Bound<T>, u: &Bound<T>) -> Result<Ordering, Status>
where
    T: Ord,
{
    let lt: &T = bound2t(l)?;
    let ut: &T = bound2t(u)?;
    Ok(lt.cmp(&ut))
}

pub fn check_bound<T>(l: &Bound<T>, u: &Bound<T>) -> Result<(), Status>
where
    T: Ord,
{
    let o: Ordering = bounds2ord(l, u)?;
    match o {
        Ordering::Less => Ok(()),
        Ordering::Equal => Ok(()),
        Ordering::Greater => Err(Status::invalid_argument("lower > upper")),
    }
}

impl Req {
    pub async fn handle_del(
        kv: &mut BTreeMap<Vec<u8>, Val>,
        req: DelRequest,
        reply: Sender<Result<DelResponse, Status>>,
    ) {
        let key: Vec<u8> = req.key;
        let res: Result<DelResponse, Status> = {
            kv.remove(&key);
            Ok(DelResponse {
                del_time: Some(SystemTime::now().into()),
            })
        };
        match reply.send(res).await {
            Ok(_) => {}
            Err(e) => error!("{e}"),
        }
    }

    pub async fn handle_range(
        kv: &BTreeMap<Vec<u8>, Val>,
        req: RangeRequest,
        reply: Sender<Receiver<Result<RangeResponse, Status>>>,
        conf: &Conf,
    ) {
        let max: usize = conf.max_range;
        let ol: Option<RBound> = req.lower;
        let ou: Option<RBound> = req.upper;
        let keys: Result<Vec<Vec<u8>>, Status> = (|| {
            let l: Bound<Vec<u8>> = bound_convert(ol)?;
            let u: Bound<Vec<u8>> = bound_convert(ou)?;
            check_bound(&l, &u)?;
            let pairs = kv.range((l, u));
            let taken = pairs.take(max);
            let keys = taken.map(|pair| pair.0).cloned();
            Ok(keys.collect())
        })();
        let (tx, rx) = tokio::sync::mpsc::channel(1);
        tokio::spawn(async move {
            let tx = &tx;
            match keys {
                Err(e) => match tx.send(Err(e.clone())).await {
                    Ok(_) => warn!("{e}"),
                    Err(e) => error!("{e}"),
                },
                Ok(v) => {
                    let mapd = v.into_iter().map(|key: Vec<u8>| Ok(RangeResponse { key }));
                    let strm = futures::stream::iter(mapd);
                    let rcnt: Result<u64, Status> = strm
                        .try_fold(0, |state, next| async move {
                            tx.send(Ok(next))
                                .await
                                .map(|_| state + 1)
                                .map_err(|e| Status::internal(format!("unable to send: {e}")))
                        })
                        .await;
                    match rcnt {
                        Ok(_) => {}
                        Err(e) => error!("{e}"),
                    }
                }
            }
        });
        match reply.send(rx).await {
            Ok(_) => {}
            Err(e) => error!("{e}"),
        }
    }
}

pub struct Conf {
    pub max_range: usize,
}

impl Default for Conf {
    fn default() -> Self {
        Self {
            max_range: MAX_RANGE_SIZE_DEFAULT,
        }
    }
}

impl Req {
    pub async fn handle(self, kv: &mut BTreeMap<Vec<u8>, Val>, conf: &Conf) {
        match self {
            Self::Set(req, reply) => Self::handle_set(kv, req, reply).await,
            Self::Get(req, reply) => Self::handle_get(kv, req, reply).await,
            Self::DSet(req, reply) => Self::handle_dset(kv, req, reply).await,
            Self::DGet(req, reply) => Self::handle_dget(kv, req, reply).await,
            Self::DHas(req, reply) => Self::handle_dhas(kv, req, reply).await,
            Self::Push(req, reply) => Self::handle_push(kv, req, reply).await,
            Self::Pop(req, reply) => Self::handle_pop(kv, req, reply).await,
            Self::QLen(req, reply) => Self::handle_qlen(kv, req, reply).await,
            Self::SAdd(req, reply) => Self::handle_sadd(kv, req, reply).await,
            Self::SDel(req, reply) => Self::handle_sdel(kv, req, reply).await,
            Self::SLen(req, reply) => Self::handle_slen(kv, req, reply).await,
            Self::Del(req, reply) => Self::handle_del(kv, req, reply).await,
            Self::Range(req, reply) => Self::handle_range(kv, req, reply, conf).await,
        }
    }
}

pub struct ChanSvc {
    sender: Sender<Req>,
}

#[tonic::async_trait]
impl MemoryDatabaseService for ChanSvc {
    type RangeStream = ReceiverStream<Result<RangeResponse, Status>>;

    async fn set(
        &self,
        request: Request<SetRequest>,
    ) -> std::result::Result<Response<SetResponse>, Status> {
        let iq: SetRequest = request.into_inner();
        let (tx, mut rx) = tokio::sync::mpsc::channel(1);
        let req = Req::Set(iq, tx);
        self.sender
            .send(req)
            .await
            .map_err(|e| Status::internal(format!("unable to send: {e}")))?;
        let ores: Option<_> = rx.recv().await;
        let rslt: Result<_, _> = ores.ok_or_else(|| Status::internal("no response got"))?;
        let res: SetResponse = rslt?;
        Ok(Response::new(res))
    }

    async fn get(
        &self,
        request: Request<GetRequest>,
    ) -> std::result::Result<Response<GetResponse>, Status> {
        let iq: GetRequest = request.into_inner();
        let (tx, mut rx) = tokio::sync::mpsc::channel(1);
        let req = Req::Get(iq, tx);
        self.sender
            .send(req)
            .await
            .map_err(|e| Status::internal(format!("unable to send: {e}")))?;
        let ores: Option<_> = rx.recv().await;
        let rslt: Result<_, _> = ores.ok_or_else(|| Status::internal("no response got"))?;
        let res: GetResponse = rslt?;
        Ok(Response::new(res))
    }

    async fn push(
        &self,
        request: Request<PushRequest>,
    ) -> std::result::Result<Response<PushResponse>, Status> {
        let iq: PushRequest = request.into_inner();
        let (tx, mut rx) = tokio::sync::mpsc::channel(1);
        let req = Req::Push(iq, tx);
        self.sender
            .send(req)
            .await
            .map_err(|e| Status::internal(format!("unable to send: {e}")))?;
        let ores: Option<_> = rx.recv().await;
        let rslt: Result<_, _> = ores.ok_or_else(|| Status::internal("no response got"))?;
        let res: PushResponse = rslt?;
        Ok(Response::new(res))
    }

    async fn pop(
        &self,
        request: Request<PopRequest>,
    ) -> std::result::Result<Response<PopResponse>, Status> {
        let iq: PopRequest = request.into_inner();
        let (tx, mut rx) = tokio::sync::mpsc::channel(1);
        let req = Req::Pop(iq, tx);
        self.sender
            .send(req)
            .await
            .map_err(|e| Status::internal(format!("unable to send: {e}")))?;
        let ores: Option<_> = rx.recv().await;
        let rslt: Result<_, _> = ores.ok_or_else(|| Status::internal("no response got"))?;
        let res: PopResponse = rslt?;
        Ok(Response::new(res))
    }

    async fn q_len(
        &self,
        request: Request<QLenRequest>,
    ) -> std::result::Result<Response<QLenResponse>, Status> {
        let iq: QLenRequest = request.into_inner();
        let (tx, mut rx) = tokio::sync::mpsc::channel(1);
        let req = Req::QLen(iq, tx);
        self.sender
            .send(req)
            .await
            .map_err(|e| Status::internal(format!("unable to send: {e}")))?;
        let ores: Option<_> = rx.recv().await;
        let rslt: Result<_, _> = ores.ok_or_else(|| Status::internal("no response got"))?;
        let res: QLenResponse = rslt?;
        Ok(Response::new(res))
    }

    async fn d_set(
        &self,
        request: Request<DSetRequest>,
    ) -> std::result::Result<Response<DSetResponse>, Status> {
        let iq: DSetRequest = request.into_inner();
        let (tx, mut rx) = tokio::sync::mpsc::channel(1);
        let req = Req::DSet(iq, tx);
        self.sender
            .send(req)
            .await
            .map_err(|e| Status::internal(format!("unable to send: {e}")))?;
        let ores: Option<_> = rx.recv().await;
        let rslt: Result<_, _> = ores.ok_or_else(|| Status::internal("no response got"))?;
        let res: DSetResponse = rslt?;
        Ok(Response::new(res))
    }

    async fn d_get(
        &self,
        request: Request<DGetRequest>,
    ) -> std::result::Result<Response<DGetResponse>, Status> {
        let iq: DGetRequest = request.into_inner();
        let (tx, mut rx) = tokio::sync::mpsc::channel(1);
        let req = Req::DGet(iq, tx);
        self.sender
            .send(req)
            .await
            .map_err(|e| Status::internal(format!("unable to send: {e}")))?;
        let ores: Option<_> = rx.recv().await;
        let rslt: Result<_, _> = ores.ok_or_else(|| Status::internal("no response got"))?;
        let res: DGetResponse = rslt?;
        Ok(Response::new(res))
    }

    async fn d_has(
        &self,
        request: Request<DHasRequest>,
    ) -> std::result::Result<Response<DHasResponse>, Status> {
        let iq: DHasRequest = request.into_inner();
        let (tx, mut rx) = tokio::sync::mpsc::channel(1);
        let req = Req::DHas(iq, tx);
        self.sender
            .send(req)
            .await
            .map_err(|e| Status::internal(format!("unable to send: {e}")))?;
        let ores: Option<_> = rx.recv().await;
        let rslt: Result<_, _> = ores.ok_or_else(|| Status::internal("no response got"))?;
        let res: DHasResponse = rslt?;
        Ok(Response::new(res))
    }
    async fn s_add(
        &self,
        request: Request<SAddRequest>,
    ) -> std::result::Result<Response<SAddResponse>, Status> {
        let iq: SAddRequest = request.into_inner();
        let (tx, mut rx) = tokio::sync::mpsc::channel(1);
        let req = Req::SAdd(iq, tx);
        self.sender
            .send(req)
            .await
            .map_err(|e| Status::internal(format!("unable to send: {e}")))?;
        let ores: Option<_> = rx.recv().await;
        let rslt: Result<_, _> = ores.ok_or_else(|| Status::internal("no response got"))?;
        let res: SAddResponse = rslt?;
        Ok(Response::new(res))
    }

    async fn s_del(
        &self,
        request: Request<SDelRequest>,
    ) -> std::result::Result<Response<SDelResponse>, Status> {
        let iq: SDelRequest = request.into_inner();
        let (tx, mut rx) = tokio::sync::mpsc::channel(1);
        let req = Req::SDel(iq, tx);
        self.sender
            .send(req)
            .await
            .map_err(|e| Status::internal(format!("unable to send: {e}")))?;
        let ores: Option<_> = rx.recv().await;
        let rslt: Result<_, _> = ores.ok_or_else(|| Status::internal("no response got"))?;
        let res: SDelResponse = rslt?;
        Ok(Response::new(res))
    }

    async fn s_len(
        &self,
        request: Request<SLenRequest>,
    ) -> std::result::Result<Response<SLenResponse>, Status> {
        let iq: SLenRequest = request.into_inner();
        let (tx, mut rx) = tokio::sync::mpsc::channel(1);
        let req = Req::SLen(iq, tx);
        self.sender
            .send(req)
            .await
            .map_err(|e| Status::internal(format!("unable to send: {e}")))?;
        let ores: Option<_> = rx.recv().await;
        let rslt: Result<_, _> = ores.ok_or_else(|| Status::internal("no response got"))?;
        let res: SLenResponse = rslt?;
        Ok(Response::new(res))
    }

    async fn del(
        &self,
        request: Request<DelRequest>,
    ) -> std::result::Result<Response<DelResponse>, Status> {
        let iq: DelRequest = request.into_inner();
        let (tx, mut rx) = tokio::sync::mpsc::channel(1);
        let req = Req::Del(iq, tx);
        self.sender
            .send(req)
            .await
            .map_err(|e| Status::internal(format!("unable to send: {e}")))?;
        let ores: Option<_> = rx.recv().await;
        let rslt: Result<_, _> = ores.ok_or_else(|| Status::internal("no response got"))?;
        let res: DelResponse = rslt?;
        Ok(Response::new(res))
    }

    async fn range(
        &self,
        request: Request<RangeRequest>,
    ) -> std::result::Result<Response<Self::RangeStream>, Status> {
        let iq: RangeRequest = request.into_inner();
        let (tx, mut rx) = tokio::sync::mpsc::channel(1);
        let req = Req::Range(iq, tx);
        self.sender
            .send(req)
            .await
            .map_err(|e| Status::internal(format!("unable to send: {e}")))?;
        let ores: Option<_> = rx.recv().await;
        let rcv: Receiver<Result<RangeResponse, Status>> =
            ores.ok_or_else(|| Status::internal("no response got"))?;
        let res: ReceiverStream<_> = ReceiverStream::new(rcv);
        Ok(Response::new(res))
    }
}

pub async fn start(mut requests: Receiver<Req>, conf: Conf) {
    let mut kv: BTreeMap<Vec<u8>, Val> = BTreeMap::new();

    loop {
        let oreq: Option<Req> = requests.recv().await;
        match oreq {
            None => return,
            Some(req) => req.handle(&mut kv, &conf).await,
        }
    }
}

pub async fn chan_svc_new(conf: Conf) -> impl MemoryDatabaseService {
    let (tx, rx) = tokio::sync::mpsc::channel(1);
    tokio::spawn(async move { start(rx, conf).await });
    ChanSvc { sender: tx }
}

pub async fn chan_svc_new_default() -> impl MemoryDatabaseService {
    chan_svc_new(Conf::default()).await
}
