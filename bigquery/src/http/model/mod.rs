#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct IterationResult {
    /// Index of the iteration, 0 based.
    pub index: i32,
    /// Time taken to run the iteration in milliseconds.
    #[serde(deserialize_with = "crate::http::from_str")]
    pub duration_ms: i64,
    /// Loss computed on the training data at the end of iteration.
    pub training_loss: f64,
    /// Loss computed on the eval data at the end of iteration.
    pub eval_loss: f64,
    /// Learn rate used for this iteration.
    pub learn_rate: f64,
    /// Information about top clusters for clustering models.
    pub cluster_infos: Vec<ClusterInfo>,
    pub arima_result: ArimaResult,
    /// The information of the principal components.
    pub principal_component_infos: Vec<PrincipalComponentInfo>,
}

#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct ClusterInfo {
    /// Centroid id.
    #[serde(deserialize_with = "crate::http::from_str")]
    pub centroid_id: i64,
    /// Cluster radius, the average distance from centroid to each point assigned to the cluster.
    pub cluster_radius: f64,
    /// Cluster size, the total number of points assigned to the cluster.
    #[serde(deserialize_with = "crate::http::from_str")]
    pub cluster_size: i64,
}

#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct ArimaResult {
    /// This message is repeated because there are multiple arima models fitted in auto-arima. For non-auto-arima model, its size is one.
    pub arima_model_info: Vec<ArimaModelInfo>,
    /// Seasonal periods. Repeated because multiple periods are supported for one time series.
    pub seasonal_periods: Vec<SeasonalPeriodType>,
}

#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct ArimaModelInfo {
    /// Non-seasonal order.
    pub non_seasonal_order: ArimaOrder,
    /// Arima coefficients.
    pub arima_coefficients: ArimaCoefficients,
    /// Arima fitting metrics.
    pub arima_fitting_metrics: ArimaFittingMetrics,
    /// Whether Arima model fitted with drift or not. It is always false when d is not 1.
    pub has_drift: bool,
    /// The timeSeriesId value for this time series.
    /// It will be one of the unique values from the timeSeriesIdColumn specified during ARIMA model training.
    /// Only present when timeSeriesIdColumn training option was used.
    pub time_series_id: String,
    /// The tuple of timeSeriesIds identifying this time series.
    /// It will be one of the unique tuples of values present in the timeSeriesIdColumns specified
    /// during ARIMA model training.
    /// Only present when timeSeriesIdColumns training option was used and
    /// the order of values here are same as the order of timeSeriesIdColumns.
    pub time_series_ids: Vec<String>,
    /// Seasonal periods. Repeated because multiple periods are supported for one time series.
    pub seasonal_periods: Vec<SeasonalPeriodType>,
    /// If true, holiday_effect is a part of time series decomposition result.
    pub has_holiday_effect: bool,
    /// If true, spikes_and_dips is a part of time series decomposition result.
    pub has_spikes_and_dips: bool,
    /// If true, step_changes is a part of time series decomposition result.
    pub has_step_changes: bool,
}

#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct ArimaCoefficients {
    /// Auto-regressive coefficients, an array of double.
    pub auto_regressive_coefficients: Vec<f64>,
    /// Moving-average coefficients, an array of double.
    pub moving_average_coefficients: Vec<f64>,
    /// Intercept coefficient, just a double not an array
    pub intercept_coefficient: f64,
}

#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct ArimaFittingMetrics {
    /// Log-likelihood.
    pub log_likelihood: f64,
    /// AIC.
    pub aic: f64,
    /// Variance.
    pub variance: f64,
}

#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SeasonalPeriodType {
    SeasonalPeriodTypeUnspecified,
    NoSeasonality,
    Daily,
    Weekly,
    Monthly,
    Quarterly,
    Yearly,
}

#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct PrincipalComponentInfo {
    /// Id of the principal component.
    #[serde(deserialize_with = "crate::http::from_str")]
    pub principal_component_id: i64,
    /// Explained variance by this principal component, which is simply the eigenvalue.
    pub explained_variance: f64,
    /// Explained_variance over the total explained variance.
    pub explained_variance_ratio: f64,
    /// The explainedVariance is pre-ordered in the descending order to compute
    /// the cumulative explained variance ratio.
    pub cumulative_explained_variance_ratio: f64,
}

#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ModelType {
    ModelTypeUnspecified,
    /// Linear regression model.
    LinearRegression,
    ///Logistic regression based classification model.
    LogisticRegression,
    /// K-means clustering model.
    Kmeans,
    /// Matrix factorization model.
    MatrixFactorization,
    /// DNN classifier model.
    DnnClassifier,
    ///An imported TensorFlow model.
    Tensorflow,
    /// DNN regressor model.
    DnnRegression,
    /// Boosted tree regressor model.
    BoostedTreeRegressor,
    /// Boosted tree classifier model.
    BoostedTreeClassifier,
    ///Arima Model
    Arima,
    /// AutoML Tables regression model.
    AutomlRegressor,
    /// AutoML Tables classification model.
    AutomlClassifier,
    /// Prinpical Component Analysis model.
    Pca,
    /// Autoencoder model.
    Autoencoder,
    /// New name for the ARIMA model.
    ArimaPlus,
}

#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct HparamTuningTrial {
    /// 1-based index of the trial.
    #[serde(deserialize_with = "crate::http::from_str")]
    pub trial_id: i64,
    /// Starting time of the trial.
    #[serde(deserialize_with = "crate::http::from_str")]
    pub start_time_ms: i64,
    /// Ending time of the trial.
    #[serde(deserialize_with = "crate::http::from_str")]
    pub end_time_ms: i64,
    /// The hyperprameters selected for this trial.
    pub hparams: TrainingOptions,
    /// Evaluation metrics of this trial calculated on the test data. Empty in Job API.
    pub evaluation_metrics: EvaluationMetrics,
    /// The status of the trial.
    pub status: TrialStatus,
    /// Error message for FAILED and INFEASIBLE trial.
    pub error_message: String,
    /// Loss computed on the training data at the end of trial.
    pub training_loss: f64,
    /// Loss computed on the eval data at the end of trial.
    pub eval_loss: f64,
    /// Hyperparameter tuning evaluation metrics of this trial calculated on the eval data
    /// . Unlike evaluationMetrics, only the fields corresponding to the hparamTuningObjectives are set.
    pub hparam_tuning_evaluation_metrics: EvaluationMetrics,
}

#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TrialStatus {
    TrialStatusUnspecified,
    NotStarted,
    Running,
    Succeeded,
    Failed,
    Infeasible,
    StoppedEarly,
}
